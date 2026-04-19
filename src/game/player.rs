use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    AppSystems, PausableSystems,
    game::combat::{Health, HealthChanged},
    screens::Screen,
};

const PLAYER_Z: f32 = 10.0;
const PLAYER_SIZE: f32 = 20.0;
const PLAYER_SPEED: f32 = 300.0;
const PLAYER_HP: f32 = 10.0;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(update_hp_bar)
        .add_systems(
            OnEnter(Screen::Gameplay),
            (spawn_health_bar, spawn_player).chain(),
        )
        .add_systems(
            Update,
            (move_player, update_follow_camera)
                .chain()
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Player;

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
        Player,
        Transform::from_xyz(0.0, 0.0, PLAYER_Z),
        Mesh2d(meshes.add(Circle::new(PLAYER_SIZE))),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::hex("#5d5dff").unwrap()))),
        RigidBody::Kinematic,
        Collider::circle(PLAYER_SIZE),
        Health::new(PLAYER_HP),
    ));
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
