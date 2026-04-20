use std::f32::consts::PI;

use avian2d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use rand::Rng;

use crate::{game::{GameRng, sonar::SonarDetectable}, screens::Screen};

const GAME_BOUNDS: Vec2 = Vec2::new(10_000.0, 10_000.0);

const FLOOR_BLOB_COUNT: usize = 5_000;
const FLOOR_BLOB_Z: f32 = 0.0;

const WALL_COUNT: usize = 500;
const WALL_Z: f32 = 10.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), (spawn_floor_blobs, spawn_walls));
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct Wall;

fn spawn_floor_blobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GameRng>,
) {
    for i in 0..FLOOR_BLOB_COUNT {
        let position = Rectangle::from_size(GAME_BOUNDS).sample_interior(&mut rng.0);
        let size = rng.0.random_range(10.0..50.0);

        commands.spawn((
            Name::new(format!("floor blob {}", i)),
            DespawnOnExit(Screen::Gameplay),
            Mesh2d(meshes.add(Circle::new(size))),
            MeshMaterial2d(materials.add(Color::Srgba(Srgba::hex("#222222ff").unwrap()))),
            Transform::from_translation(position.extend(FLOOR_BLOB_Z)),
        ));
    }
}

fn spawn_walls(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rng: ResMut<GameRng>,
) {
    for i in 0..WALL_COUNT {
        let position;
        loop {
            let sampled_position = Rectangle::from_size(GAME_BOUNDS).sample_interior(&mut rng.0);

            if sampled_position.distance_squared(Vec2::ZERO) >= 300.0 * 300.0 {
                position = sampled_position;
                break;
            }
        }

        let width = rng.0.random_range(80.0..300.0);
        let height = rng.0.random_range(80.0..300.0);
        let angle = rng.0.random_range(0.0..2.0 * PI);

        commands.spawn((
            Name::new(format!("wall {}", i)),
            Wall,
            DespawnOnExit(Screen::Gameplay),
            Mesh2d(meshes.add(Rectangle::new(width, height).to_ring(5.0))),
            MeshMaterial2d(materials.add(Color::Srgba(Srgba::hex("#ffff00ff").unwrap()))),
            Transform::from_translation(position.extend(WALL_Z))
                .with_rotation(Quat::from_rotation_z(angle)),
            SonarDetectable::from_radius(width.min(height)),
            Visibility::Hidden,
            RigidBody::Static,
            Collider::rectangle(width, height),
        ));
    }
}
