use bevy::prelude::*;
use bevy_prototype_debug_lines::{DebugLinesPlugin, DebugLinesSet};

pub mod components;
pub mod systems;

#[derive(Default, Clone, Hash, PartialEq, Eq, Debug, States)]
enum IsDebugDraw {
    Off,
    #[default]
    On,
}

#[derive(Resource)]
pub struct SpacePhysicsDraw {

}


pub struct SpacePhysicsDebugDrawPlugin;


impl Plugin for SpacePhysicsDebugDrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<IsDebugDraw>();

        app.add_plugin(DebugLinesPlugin::with_depth_test(true));

        app.add_systems(Update, systems::draw_colliders.run_if(in_state(IsDebugDraw::On)).before(DebugLinesSet::DrawLines));
    }
}



