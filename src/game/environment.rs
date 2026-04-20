use bevy::prelude::*;
use rand::Rng;

use crate::{game::GameRng, screens::Screen};

const GAME_BOUNDS: Vec2 = Vec2::new(10_000.0, 10_000.0);

const FLOOR_BLOB_COUNT: usize = 5_000;
const FLOOR_BLOB_Z: f32 = 0.0;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Gameplay), spawn_floor_blobs);
}

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
            Transform::from_translation(position.extend(FLOOR_BLOB_Z)),
            Mesh2d(meshes.add(Circle::new(size))),
            MeshMaterial2d(materials.add(Color::Srgba(Srgba::hex("#222222ff").unwrap()))),
        ));
    }
}
