use space_assets::Location;
use crate::*;
use space_core::ecs::*;

fn update_loc_buffer(mut query: Query<&mut Location>) {
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

    fn add_system(&self, game: &mut Game, builder:  &mut space_core::ecs::Schedule) {
        builder.add_system_to_stage(GlobalStageStep::Logic,update_loc_buffer);
    }
}

