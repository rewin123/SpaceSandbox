pub mod pilot_seat;
pub mod meteor;

pub mod prelude {
    pub use super::pilot_seat::*;
    pub use super::meteor::*;
    pub use super::*;
}

use bevy::prelude::*;

pub struct SpaceObjectsPlugin;

impl Plugin for SpaceObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(pilot_seat::PilotSeatPlugin);
        app.add_plugin(meteor::MetorFieldPlugin);
    }
}