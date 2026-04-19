use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, screens::Screen};

const PLAYER_Z: f32 = 10.0;
const PLAYER_SIZE: f32 = 20.0;
const PLAYER_SPEED: f32 = 300.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_player)
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
    ));

    commands.spawn((
        Name::new("test circle"),
        DespawnOnExit(Screen::Gameplay),
        Transform::from_xyz(128.0, 0.0, PLAYER_Z),
        Mesh2d(meshes.add(Circle::new(48.0))),
        MeshMaterial2d(materials.add(Color::Srgba(Srgba::hex("#ffffffff").unwrap()))),
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
