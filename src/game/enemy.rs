use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::game::{GameRng, player::Player};

const ENEMY_SIZE: f32 = 20.0;
const ENEMY_Z: f32 = 20.0;
const OUTLINE_THICKNESS: f32 = 5.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<EnemyAssets>()
        .add_systems(Update, spawn_enemy.run_if(input_just_pressed(KeyCode::F2)));
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

fn spawn_enemy(
    mut commands: Commands,
    enemy_assets: Res<EnemyAssets>,
    mut rng: ResMut<GameRng>,
    player_transform: Single<&Transform, With<Player>>,
) {
    const MIN_DISTANCE: f32 = 300.0;
    const MAX_DISTANCE: f32 = 700.0;

    let offset = Annulus::new(MIN_DISTANCE, MAX_DISTANCE).sample_interior(&mut rng.0);
    let position = player_transform.translation.xy() + offset;

    commands.spawn((
        Name::new("Enemy"),
        Mesh2d(enemy_assets.mesh.clone()),
        MeshMaterial2d(enemy_assets.material.clone()),
        Transform::from_translation(position.extend(ENEMY_Z)),
    ));
}
