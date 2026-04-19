use bevy::prelude::*;
use rand::{SeedableRng, rngs::StdRng};

mod combat;
mod enemy;
mod player;
mod sonar;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<GameRng>().add_plugins((
        combat::plugin,
        enemy::plugin,
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
