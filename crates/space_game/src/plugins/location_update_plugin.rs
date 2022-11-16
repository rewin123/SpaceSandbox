use space_assets::Location;
use crate::*;
use space_core::ecs::*;
use bevy::prelude::*;

fn update_loc_buffer(mut query: Query<&mut Location, Changed<Location>>) {
    for mut loc in &mut query {
        loc.update_buffer();
    }
}


pub struct LocUpdateSystem {

}

impl SchedulePlugin for LocUpdateSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("LocUpdateSystem".into())
    }

    fn add_system(&self, app:  &mut space_core::app::App) {
        app.add_system_to_stage(CoreStage::Update,update_loc_buffer);
    }
}

