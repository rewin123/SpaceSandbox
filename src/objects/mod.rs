pub mod pilot_seat;
pub mod meteor;
pub mod radar;
pub mod door;
pub mod ship_camera;

pub mod prelude {
    pub use super::pilot_seat::*;
    pub use super::meteor::*;
    pub use super::*;
    pub use radar::*;
    pub use door::*;
}

use bevy::prelude::*;

pub struct SpaceObjectsPlugin;

impl Plugin for SpaceObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(pilot_seat::PilotSeatPlugin);
        app.add_plugin(meteor::MetorFieldPlugin);
        app.add_plugin(radar::RadarPlugin);
        app.add_plugin(door::DoorPlugin);
        app.register_type::<ship_camera::ShipCamera>();
    }
}