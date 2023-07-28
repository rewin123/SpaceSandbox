pub mod hierarchy; 
pub mod selected;

use bevy::prelude::*;

pub struct SpaceEditorPlugin {

}

impl Default for SpaceEditorPlugin {
    fn default() -> Self {
        Self {

        }
    }
}

impl Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_egui::EguiPlugin);
        app.add_plugins(selected::SelectedPlugin);
        app.add_plugins(hierarchy::SpaceHierarchyPlugin::default());
    }
}