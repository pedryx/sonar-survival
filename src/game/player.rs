use std::f32::consts::FRAC_PI_2;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::{
        GameLayer, GameStats, combat::{ContactDamage, Death, DespawnOnDamageDealt, DespawnOnDeath, Health, HealthChanged}
    },
    screens::Screen,
};

const PLAYER_Z: f32 = 10.0;
const PLAYER_SIZE: f32 = 20.0;
const PLAYER_SPEED: f32 = 300.0;
const PLAYER_HP: f32 = 10.0;

const SHOOTING_COOLDOWN: f32 = 0.5;
const BULLET_RADIUS: f32 = 1.0;
const BULLET_LENGTH: f32 = 9.0;
const BULLET_Z: f32 = 30.0;
const BULLET_DAMAGE: f32 = 1.0;
const BULLET_SPEED: f32 = 600.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<BulletAssets>()
        .add_observer(update_hp_bar)
        .add_observer(on_player_death)
        .add_systems(
            OnEnter(Screen::Gameplay),
            (spawn_health_bar, spawn_player).chain(),
        )
        .add_systems(
            Update,
            ((move_player, update_follow_camera).chain(), fire_bullet)
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Player {
    shooting_cooldown_timer: Timer,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            shooting_cooldown_timer: Timer::from_seconds(SHOOTING_COOLDOWN, TimerMode::Once),
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct HealthBar;

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Name::new("player"),
        DespawnOnExit(Screen::Gameplay),
        Player::default(),
        Transform::from_xyz(0.0, 0.0, PLAYER_Z),
        Mesh2d(meshes.add(Circle::new(PLAYER_SIZE))),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::hex("#5d5dff").unwrap()))),
        RigidBody::Kinematic,
        Collider::circle(PLAYER_SIZE),
        Health::new(PLAYER_HP),
        GameLayer::Player,
        DespawnOnDeath,
    ));
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct BulletAssets {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
}

impl FromWorld for BulletAssets {
    fn from_world(world: &mut World) -> Self {
        Self {
            mesh: world
                .resource_mut::<Assets<Mesh>>()
                .add(Capsule2d::new(BULLET_RADIUS, BULLET_LENGTH)),
            material: world
                .resource_mut::<Assets<ColorMaterial>>()
                .add(Color::Srgba(Srgba::hex("#22ff00ff").unwrap())),
        }
    }
}

fn spawn_health_bar(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("hp bar"),
        DespawnOnExit(Screen::Gameplay),
        HealthBar,
        Text::new("HP: XX/YY"),
        TextFont {
            font: asset_server.load("fonts/ethnocentric_regular.otf"),
            font_size: 52.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        TextColor(Color::Srgba(Srgba::hex("#ff0000").unwrap())),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(3),
            right: px(7),
            ..default()
        },
    ));
}

fn update_hp_bar(
    _: On<HealthChanged>,
    player_health: Single<&Health, With<Player>>,
    mut health_bar_text: Single<&mut Text, With<HealthBar>>,
) {
    **health_bar_text = Text::new(format!(
        "HP: {}/{}",
        player_health.current, player_health.max
    ));
}

fn update_follow_camera(
    player: Single<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    camera.translation = player.translation.with_z(camera.translation.z);
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut velocity: Single<&mut LinearVelocity, With<Player>>,
) {
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    let direction = direction.normalize_or_zero();

    velocity.0 = direction * PLAYER_SPEED;
}

fn fire_bullet(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    bullet_assets: Res<BulletAssets>,
    player: Single<(&mut Player, &Transform)>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
) {
    let (mut player, player_transform) = player.into_inner();
    player.shooting_cooldown_timer.tick(time.delta());

    if !mouse.pressed(MouseButton::Left) || !player.shooting_cooldown_timer.is_finished() {
        return;
    }
    player.shooting_cooldown_timer.reset();

    let (camera, camera_transform) = *camera;
    let Some(viewport_position) = window.cursor_position() else {
        return;
    };
    let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, viewport_position)
    else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let direction = (world_position - player_pos).normalize_or_zero();
    let angle = direction.to_angle();

    commands.spawn((
        Name::new("Bullet"),
        DespawnOnExit(Screen::Gameplay),
        Mesh2d(bullet_assets.mesh.clone()),
        MeshMaterial2d(bullet_assets.material.clone()),
        Transform::from_translation(player_pos.extend(BULLET_Z))
            .with_rotation(Quat::from_rotation_z(angle - FRAC_PI_2)),
        RigidBody::Kinematic,
        Collider::capsule(BULLET_RADIUS, BULLET_LENGTH),
        LinearVelocity(direction * BULLET_SPEED),
        ContactDamage::new(BULLET_DAMAGE, GameLayer::Enemy, 0.0),
        DespawnOnDamageDealt,
    ));
}

fn on_player_death(
    event: On<Death>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_stats: ResMut<GameStats>,
    query: Query<(), With<Player>>,
) {
    if query.get(event.0).is_err() {
        return;
    }

    commands.spawn((
        Name::new("game over ui"),
        DespawnOnExit(Screen::Gameplay),
        Text::new("You Died!"),
        TextFont {
            font: asset_server.load("fonts/ethnocentric_regular.otf"),
            font_size: 86.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        TextColor(Color::Srgba(Srgba::hex("#ff0000ff").unwrap())),
        Node {
            position_type: PositionType::Absolute,
            top: px(300),
            bottom: px(0),
            left: px(0),
            right: px(0),
            ..default()
        },
    ));

    game_stats.tracking_enabled = false;
}
