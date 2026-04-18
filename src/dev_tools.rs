//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions,
    input::common_conditions::{input_just_pressed, input_toggle_active},
    prelude::*,
};
use bevy_console::ConsolePlugin;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
    // Log `Screen` state transitions.
    app.add_systems(Update, log_transitions::<Screen>);

    // Toggle the debug overlay for UI.
    app.add_systems(
        Update,
        toggle_debug_ui.run_if(input_just_pressed(TOGGLE_KEY)),
    );

    app.add_plugins(EguiPlugin::default()).add_plugins(
        WorldInspectorPlugin::new()
            .run_if(input_toggle_active(false, TOGGLE_KEY))
            .run_if(input_toggle_active(false, CONSOLE_KEY)),
    );

    app.add_plugins(ConsolePlugin);
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;
const CONSOLE_KEY: KeyCode = KeyCode::F1;

fn toggle_debug_ui(mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
