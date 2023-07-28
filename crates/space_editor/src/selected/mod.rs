use bevy::{prelude::*, utils::HashSet};

#[derive(Resource, Default)]
pub struct SelectedEntities {
    pub list : HashSet<Entity>
}

pub struct SelectedPlugin;

impl Plugin for SelectedPlugin {
    fn build(&self, app : &mut App) {
        app.init_resource::<SelectedEntities>();
    }
}

