use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{AppSystems, PausableSystems};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(despawn_dead).add_systems(
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
#[require(Collider, CollidingEntities)]
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

#[derive(Event, Debug)]
pub struct Died(pub Entity);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct DestroyOnDeath;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(Collider)]
pub struct ContactDamage {
    pub damage: f32,
    cooldown_timer: Timer,
}

impl ContactDamage {
    pub fn new(damage: f32, cooldown_secs: f32) -> Self {
        let mut cooldown_timer = Timer::from_seconds(cooldown_secs, TimerMode::Once);
        cooldown_timer.tick(cooldown_timer.duration());

        Self {
            damage,
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

fn despawn_dead(event: On<Died>, mut commands: Commands) {
    commands.entity(event.0).despawn();
}

fn update_damage_cooldown(time: Res<Time>, damagers: Query<&mut ContactDamage>) {
    for mut contact_damage in damagers {
        contact_damage.cooldown_timer.tick(time.delta());
    }
}

fn apply_contact_damage(
    query: Query<(&mut Health, &CollidingEntities, &Name)>,
    mut damagers: Query<&mut ContactDamage>,
) {
    for (mut health, colliding_entities, name) in query {
        if health.is_dead {
            continue;
        }

        for &entity in colliding_entities.iter() {
            let Ok(mut contact_damage) = damagers.get_mut(entity) else {
                continue;
            };

            if !contact_damage.cooldown_timer.is_finished() {
                continue;
            }

            contact_damage.cooldown_timer.reset();
            health.current -= contact_damage.damage;

            info!(
                "{} took contact damage, hp: {}/{}",
                name, health.current, health.max
            );
        }
    }
}
