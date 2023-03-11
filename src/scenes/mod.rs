pub mod station_builder;
pub mod fps_mode;
pub mod main_menu;
pub mod settings;

use bevy::prelude::*;

pub struct NotificationPlugin;

use bevy_egui::EguiContext;
use egui_notify::*;

#[derive(Resource, Default)]
pub struct ToastHolder {
    pub toast : Toasts
}


impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ToastHolder::default());
        app.add_system(show_toasts);
    }
}

fn show_toasts(
    mut ctx : Query<&mut EguiContext>,
    mut toasts : ResMut<ToastHolder>,
) {
    toasts.toast.show(&ctx.single_mut().get_mut());
}