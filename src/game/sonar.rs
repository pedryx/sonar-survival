use bevy::prelude::*;

use crate::{AppSystems, PausableSystems, game::player::Player, screens::Screen};

const WAVE_THICKNESS: f32 = 5.0;
const WAVE_PADDING: f32 = 10.0;
const WAVE_SPEED: f32 = 500.0;
const WAVE_COUNT: usize = 4;
const WAVE_MAX_RADIUS: f32 = 1100.0;

const SONAR_Z: f32 = 100.0;
const SONAR_PERIOD_SECS: f32 = 5.0;

const OUTLINE_DURATION: f32 = 3.0;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<WaveMaterial>()
        .init_resource::<WaveSpawner>()
        .add_systems(
            Update,
            (
                (spawn_waves, propagate_waves).chain(),
                show_outlines,
                hide_outlines,
            )
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        );
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct WaveMaterial(Handle<ColorMaterial>);

impl FromWorld for WaveMaterial {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<ColorMaterial>>();
        let color = materials.add(Color::Srgba(Srgba::hex("#00fff22a").unwrap()));

        Self(color)
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
struct WaveSpawner(Timer);

impl Default for WaveSpawner {
    fn default() -> Self {
        Self(Timer::from_seconds(SONAR_PERIOD_SECS, TimerMode::Repeating))
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct SonarDetectable {
    outline_timer: Timer,
    pub radius: f32,
}

impl SonarDetectable {
    pub fn from_radius(radius: f32) -> Self {
        Self {
            outline_timer: Timer::from_seconds(OUTLINE_DURATION, TimerMode::Once),
            radius,
        }
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct SonarWave {
    radius: f32,
}

fn spawn_waves(
    mut commands: Commands,
    time: Res<Time>,
    sonar_visuals: Res<WaveMaterial>,
    mut wave_spawner: ResMut<WaveSpawner>,
    mut meshes: ResMut<Assets<Mesh>>,
    player_transform: Single<&Transform, With<Player>>,
) {
    wave_spawner.0.tick(time.delta());
    if !wave_spawner.0.just_finished() {
        return;
    }

    for i in 0..WAVE_COUNT {
        let radius = (i as f32) * (WAVE_THICKNESS + WAVE_PADDING);

        commands.spawn((
            Name::new(format!("Sonar Wave {}", i)),
            DespawnOnExit(Screen::Gameplay),
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
            .resolution(512)
            .into()
    }
}

fn show_outlines(
    waves: Query<(&SonarWave, &Transform)>,
    mut detectables: Query<(&mut SonarDetectable, &mut Visibility, &Transform)>,
) {
    for (wave, wave_transform) in waves {
        for (mut detectable, mut visibility, detectable_transform) in detectables.iter_mut() {
            let wave_position = wave_transform.translation.xy();
            let detectable_position = detectable_transform.translation.xy();

            let distance = wave_position.distance(detectable_position);
            let inner = wave.radius - WAVE_THICKNESS;
            let outer = wave.radius;

            if distance + detectable.radius >= inner && distance - detectable.radius <= outer {
                detectable.outline_timer.reset();
                *visibility = Visibility::Visible;
            }
        }
    }
}

fn hide_outlines(time: Res<Time>, query: Query<(&mut SonarDetectable, &mut Visibility)>) {
    for (mut detectable, mut visibility) in query {
        detectable.outline_timer.tick(time.delta());

        if detectable.outline_timer.just_finished() {
            *visibility = Visibility::Hidden;
        }
    }
}
