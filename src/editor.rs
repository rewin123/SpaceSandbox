use bevy::{prelude::*, asset::HandleId};
use bevy_egui::egui::util::id_type_map::TypeId;
use bevy_inspector_egui::{prelude::*, *, bevy_inspector::hierarchy::SelectedEntities, quick::WorldInspectorPlugin};
use egui_dock::Tree;
use egui_gizmo::GizmoMode;
pub struct EditorPlugin;


impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DefaultInspectorConfigPlugin);
        app.add_plugin(WorldInspectorPlugin::new());
    }
}

