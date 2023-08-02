pub mod gizmo_move;

use bevy::{prelude::*, utils::HashSet};

#[derive(Resource, Default, Clone)]
pub struct SelectedEntities {
    pub list : HashSet<Entity>
}

pub struct SelectedPlugin;

impl Plugin for SelectedPlugin {
    fn build(&self, app : &mut App) {
        app.init_resource::<SelectedEntities>();

        app.add_plugins(gizmo_move::SpaceGizmoPlugin);
    }
}

