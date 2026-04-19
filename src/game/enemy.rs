use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        GameLayer, GameRng,
        combat::{ContactDamage, DespawnOnDeath, Health},
        player::Player,
        sonar::SonarDetectable,
    },
    screens::Screen,
};

const ENEMY_SIZE: f32 = 20.0;
const ENEMY_Z: f32 = 20.0;
const ENEMY_SPEED: f32 = 80.0;
const ENEMY_DAMAGE: f32 = 1.0;
const ENEMY_DAMAGE_COOLDOWN_SECS: f32 = 1.0;
const ENEMY_HP: f32 = 1.0;
const OUTLINE_THICKNESS: f32 = 5.0;

const SPAWN_PERIOD_SECS: f32 = 2.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EnemySpawner>()
        .init_resource::<EnemyAssets>()
        .add_systems(
            Update,
            (spawn_enemy, follow_player)
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct EnemySpawner(Timer);

impl Default for EnemySpawner {
    fn default() -> Self {
        Self(Timer::from_seconds(SPAWN_PERIOD_SECS, TimerMode::Repeating))
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct EnemyAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

impl FromWorld for EnemyAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Circle::new(ENEMY_SIZE).to_ring(OUTLINE_THICKNESS)),
            material: world
                .resource_mut::<Assets<ColorMaterial>>()
                .add(Color::Srgba(Srgba::hex("#ff7575ff").unwrap())),
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Enemy;

fn spawn_enemy(
    mut commands: Commands,
    time: Res<Time>,
    mut enemy_spawner: ResMut<EnemySpawner>,
    enemy_assets: Res<EnemyAssets>,
    mut rng: ResMut<GameRng>,
    player_transform: Single<&Transform, With<Player>>,
) {
    enemy_spawner.0.tick(time.delta());
    if !enemy_spawner.0.just_finished() {
        return;
    }

    const MIN_DISTANCE: f32 = 300.0;
    const MAX_DISTANCE: f32 = 700.0;

    let offset = Annulus::new(MIN_DISTANCE, MAX_DISTANCE).sample_interior(&mut rng.0);
    let position = player_transform.translation.xy() + offset;

    commands.spawn((
        Name::new("Enemy"),
        DespawnOnExit(Screen::Gameplay),
        Enemy,
        Mesh2d(enemy_assets.mesh.clone()),
        MeshMaterial2d(enemy_assets.material.clone()),
        Transform::from_translation(position.extend(ENEMY_Z)),
        SonarDetectable::from_radius(ENEMY_SIZE),
        Visibility::Hidden,
        RigidBody::Dynamic,
        Collider::circle(ENEMY_SIZE),
        ContactDamage::new(ENEMY_DAMAGE, GameLayer::Player, ENEMY_DAMAGE_COOLDOWN_SECS),
        Health::new(ENEMY_HP),
        GameLayer::Enemy,
        DespawnOnDeath,
    ));
}

fn follow_player(
    player_transform: Single<&Transform, With<Player>>,
    enemies: Query<(&mut LinearVelocity, &Transform), With<Enemy>>,
) {
    let player_position = player_transform.translation.xy();

    for (mut velocity, transform) in enemies {
        let enemy_position = transform.translation.xy();
        let direction = (player_position - enemy_position).normalize_or_zero();

        velocity.0 = direction * ENEMY_SPEED;
    }
}
