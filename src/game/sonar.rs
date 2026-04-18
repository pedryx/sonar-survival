use bevy::{input::common_conditions::input_just_pressed, prelude::*};

use crate::{AppSystems, PausableSystems, game::player::Player};

const WAVE_THICKNESS: f32 = 5.0;
const WAVE_PADDING: f32 = 10.0;
const WAVE_SPEED: f32 = 200.0;
const WAVE_COUNT: usize = 4;
const WAVE_MAX_RADIUS: f32 = 1100.0;
const SONAR_Z: f32 = 100.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<SonarMaterial>()
        .add_systems(Update, spawn_waves.run_if(input_just_pressed(KeyCode::F2)))
        .add_systems(
            Update,
            propagate_waves
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct SonarMaterial(Handle<ColorMaterial>);

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct SonarWave {
    radius: f32,
}

impl FromWorld for SonarMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let color = materials.add(Color::Srgba(Srgba::hex("#00fff22a").unwrap()));

        Self(color)
    }
}

fn spawn_waves(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    player_transform: Single<&Transform, With<Player>>,
    sonar_visuals: Res<SonarMaterial>,
) {
    for i in 0..WAVE_COUNT {
        let radius = (i as f32) * (WAVE_THICKNESS + WAVE_PADDING);

        commands.spawn((
            Name::new(format!("Sonar Wave {}", i)),
            SonarWave { radius },
            Transform::from_translation(player_transform.translation.with_z(SONAR_Z)),
            MeshMaterial2d(sonar_visuals.0.clone()),
            Mesh2d(meshes.add(Circle::new(radius).to_ring(WAVE_THICKNESS))),
        ));
    }

    info!("sonar waves spawned");
}

fn propagate_waves(
    mut commands: Commands,
    time: Res<Time>,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(Entity, &mut SonarWave, &Mesh2d)>,
) {
    for (entity, mut wave, mesh) in query {
        wave.radius += WAVE_SPEED * time.delta_secs();

        if wave.radius >= WAVE_MAX_RADIUS {
            meshes.remove(&mesh.0);
            commands.entity(entity).despawn();
            continue;
        }

        *meshes.get_mut(&mesh.0).unwrap() = Annulus::new(wave.radius, wave.radius + WAVE_THICKNESS)
            .mesh()
            .resolution(128)
            .into()
    }
}
