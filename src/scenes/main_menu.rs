use bevy::prelude::*;
use bevy_egui::*;
use crate::*;

fn main_menu(
    mut cmds : Commands,
    mut egui_context: Query<&mut EguiContext>,
    mut next_scene : ResMut<NextState<SceneType>>
) {
    egui::Window::new("Space sandbox")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(egui_context.single_mut().get_mut(), |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("Play mission").clicked() {

                }
                if ui.button("Station builder").clicked() {
                    next_scene.set(SceneType::ShipBuilding);
                }
                if ui.button("Asset editor").clicked() {
                    next_scene.set(SceneType::AssetEditor);
                }
                if ui.button("Exit").clicked() {
                    // cmds.push(GameCommands::Exit);
                }
            });
    });
}

// fn ui_example(mut egui_context: ResMut<EguiContext>) {
//     egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
//         ui.label("world");
//     });
// }

pub struct MainMenuPlugin {

}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SceneType>();
        app.add_state::<Gamemode>();

        app.add_system(main_menu.in_set(OnUpdate(SceneType::MainMenu)));
    }
}