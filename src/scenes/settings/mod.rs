
use bevy::prelude::*;
use bevy_egui::*;

use crate::control::KeyMapperWindow;

#[derive(Resource, Default)]
pub struct Settings {
    pub show_controls : bool,
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Settings::default());
        app.add_system(settings_system);
    }
}


pub fn settings_system(
    mut settings : ResMut<Settings>,
    mut key_mapper : ResMut<KeyMapperWindow>,
    mut egui_ctxs : Query<&mut EguiContext>) {
    
    egui::TopBottomPanel::top("top_panel").show(egui_ctxs.single_mut().get_mut(), |ui| {
        if ui.button("Control settings").clicked() {
            key_mapper.is_shown = !key_mapper.is_shown;
        }
    });
}
