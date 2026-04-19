use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, game::GameLayer};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(|event: On<Add, Health>, mut commands: Commands| {
        commands.trigger(HealthChanged(event.entity));
    })
    .add_observer(despawn_dead)
    .add_systems(
        Update,
        (
            (update_damage_cooldown, apply_contact_damage).chain(),
            kill_entities,
        )
            .chain()
            .in_set(AppSystems::Update)
            .in_set(PausableSystems),
    );
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Collider, CollidingEntities, GameLayer)]
pub struct Health {
    pub max: f32,
    pub current: f32,
    is_dead: bool,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            max,
            current: max,
            is_dead: false,
        }
    }
}

#[derive(EntityEvent, Debug)]
pub struct HealthChanged(pub Entity);

#[derive(EntityEvent, Debug)]
pub struct Died(pub Entity);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct DespawnOnDeath;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct DespawnOnDamageDealt;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Collider)]
pub struct ContactDamage {
    pub damage: f32,
    pub layer: GameLayer,
    cooldown_timer: Timer,
}

impl ContactDamage {
    pub fn new(damage: f32, layer: GameLayer, cooldown_secs: f32) -> Self {
        let mut cooldown_timer = Timer::from_seconds(cooldown_secs, TimerMode::Once);
        cooldown_timer.tick(cooldown_timer.duration());

        Self {
            damage,
            layer,
            cooldown_timer,
        }
    }
}

fn kill_entities(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query {
        if health.is_dead || health.current > 0.0 {
            continue;
        }

        commands.trigger(Died(entity));
    }
}

fn despawn_dead(event: On<Died>, query: Query<(), With<DespawnOnDeath>>, mut commands: Commands) {
    if query.get(event.0).is_ok() {
        commands.entity(event.0).despawn();
    }
}

fn update_damage_cooldown(time: Res<Time>, damagers: Query<&mut ContactDamage>) {
    for mut contact_damage in damagers {
        contact_damage.cooldown_timer.tick(time.delta());
    }
}

fn apply_contact_damage(
    mut commands: Commands,
    receivers: Query<(Entity, &mut Health, &CollidingEntities, &Name, &GameLayer)>,
    mut attackers: Query<(&mut ContactDamage, Option<&DespawnOnDamageDealt>)>,
) {
    for (receiver_entity, mut receiver_health, colliding_entities, receiver_name, receiver_layer) in
        receivers
    {
        if receiver_health.is_dead {
            continue;
        }

        for &attacker_entity in colliding_entities.iter() {
            let Ok((mut attacker_damage, despawn_on_damage_dealt)) =
                attackers.get_mut(attacker_entity)
            else {
                continue;
            };

            if *receiver_layer != attacker_damage.layer {
                continue;
            }

            if !attacker_damage.cooldown_timer.is_finished() {
                continue;
            }

            attacker_damage.cooldown_timer.reset();
            receiver_health.current -= attacker_damage.damage;

            info!(
                "{} took contact damage, hp: {}/{}",
                receiver_name, receiver_health.current, receiver_health.max
            );

            commands.trigger(HealthChanged(receiver_entity));

            if despawn_on_damage_dealt.is_some() {
                commands.entity(attacker_entity).despawn();
            }
        }
    }
}
