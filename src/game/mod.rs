use bevy::prelude::*;
use rand::{SeedableRng, rngs::StdRng};

use crate::{
    AppSystems, PausableSystems,
    game::{combat::Death, enemy::Enemy},
    screens::Screen,
};

mod combat;
mod enemy;
mod environment;
mod player;
mod sonar;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameRng>()
        .init_resource::<GameStats>()
        .add_observer(update_kill_count)
        .add_systems(OnEnter(Screen::Gameplay), spawn_stats_ui)
        .add_systems(
            Update,
            update_elapsed_time
                .run_if(in_state(Screen::Gameplay))
                .in_set(AppSystems::Update)
                .in_set(PausableSystems),
        )
        .add_plugins((
            combat::plugin,
            enemy::plugin,
            environment::plugin,
            player::plugin,
            sonar::plugin,
        ));
}

#[derive(Resource, Debug)]
pub struct GameRng(StdRng);

impl Default for GameRng {
    fn default() -> Self {
        Self(StdRng::seed_from_u64(0xDEAD_C0DE))
    }
}

#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct GameStats {
    pub kill_count: u32,
    pub elapsed_time_secs: f32,
    pub tracking_enabled: bool,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            kill_count: 0,
            elapsed_time_secs: 0.0,
            tracking_enabled: true,
        }
    }
}

#[derive(Component, Reflect, Debug, Default, PartialEq)]
#[reflect(Component)]
pub enum GameLayer {
    #[default]
    Environment,
    Player,
    Enemy,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct KillCountUI;

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct TimerUI;

fn spawn_stats_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Name::new("kill count ui"),
        DespawnOnExit(Screen::Gameplay),
        KillCountUI,
        Text::new("0"),
        TextFont {
            font: asset_server.load("fonts/ethnocentric_regular.otf"),
            font_size: 52.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        TextColor(Color::Srgba(Srgba::hex("#aa00ffff").unwrap())),
        Node {
            position_type: PositionType::Absolute,
            top: px(3),
            left: px(7),
            ..default()
        },
    ));

    commands.spawn((
        Name::new("timer ui"),
        DespawnOnExit(Screen::Gameplay),
        TimerUI,
        Text::new("00:00"),
        TextFont {
            font: asset_server.load("fonts/ethnocentric_regular.otf"),
            font_size: 52.0,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Center),
        TextColor(Color::Srgba(Srgba::hex("#abababff").unwrap())),
        Node {
            position_type: PositionType::Absolute,
            top: px(3),
            left: px(0),
            right: px(0),
            ..default()
        },
    ));
}

fn update_elapsed_time(
    time: Res<Time>,
    mut game_stats: ResMut<GameStats>,
    mut timer_ui: Single<&mut Text, With<TimerUI>>,
) {
    if !game_stats.tracking_enabled {
        return;
    }

    game_stats.elapsed_time_secs += time.delta_secs();

    let total_seconds = game_stats.elapsed_time_secs as u32;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    timer_ui.0 = format!("{:02}:{:02}", minutes, seconds);
}

fn update_kill_count(
    event: On<Death>,
    mut game_stats: ResMut<GameStats>,
    enemies: Query<(), With<Enemy>>,
    mut kill_ui: Single<&mut Text, With<KillCountUI>>,
) {
    if !game_stats.tracking_enabled {
        return;
    }

    if enemies.get(event.0).is_ok() {
        game_stats.kill_count += 1;
        kill_ui.0 = game_stats.kill_count.to_string();
    }
}
