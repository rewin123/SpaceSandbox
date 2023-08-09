pub mod station_builder;
pub mod fps_mode;
pub mod main_menu;
pub mod settings;
pub mod asset_editor;

use bevy::prelude::*;

pub struct NotificationPlugin;

use bevy_egui::{EguiContext, EguiContexts};
use egui_notify::*;

#[derive(Resource, Default)]
pub struct ToastHolder {
    pub toast : Toasts
}


impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ToastHolder::default());
        app.add_systems(Update, show_toasts);
    }
}

fn show_toasts(
    mut ctx : EguiContexts,
    mut toasts : ResMut<ToastHolder>,
) {
    // toasts.toast.show(ctx.ctx_mut());
}