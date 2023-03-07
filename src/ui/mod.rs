mod main_menu;

use bevy_egui::EguiContext;
pub use main_menu::*;

use bevy::prelude::*;

pub struct NotificationPlugin;

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
    mut ctx : Query<&EguiContext>,
    mut toasts : ResMut<ToastHolder>,
) {
    toasts.toast.show(&ctx.single().0);
}
