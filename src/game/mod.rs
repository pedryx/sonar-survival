use bevy::prelude::*;

mod player;
mod sonar;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((player::plugin, sonar::plugin));
}
