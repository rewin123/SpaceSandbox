use space_assets::Location;
use crate::{Game, PluginName, PluginType, SchedulePlugin};
use legion::*;
use legion::systems::Builder;

#[system(for_each)]
fn update_loc_buffer(loc : &mut Location) {
    loc.update_buffer();
}


pub struct LocUpdateSystem {

}

impl SchedulePlugin for LocUpdateSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("LocUpdateSystem".into())
    }

    fn get_plugin_type(&self) -> PluginType {
        PluginType::RenderPrepare
    }

    fn add_system(&self, game: &mut Game, builder: &mut Builder) {
        builder.add_system(update_loc_buffer_system());
    }
}

