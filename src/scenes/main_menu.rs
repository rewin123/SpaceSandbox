use bevy::prelude::*;
use bevy_egui::{*, egui::{TextureHandle, TextureId}};
use crate::*;

fn main_menu(
    mut cmds : Commands,
    mut egui_context: Query<&mut EguiContext>,
    mut next_scene : ResMut<NextState<SceneType>>,
    background : ResMut<BackgroundImage>,
) {
    egui::CentralPanel::default().show(egui_context.single_mut().get_mut(), |ui| {
        let size = ui.available_size();
        ui.image(background.0, size);
    });
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

#[derive(Resource)]
struct BackgroundImage(TextureId, Handle<Image>);

pub struct MainMenuPlugin {

}

fn setup_main_menu(
    mut commands: Commands, 
    asset_server : ResMut<AssetServer>,
    mut egui_context: EguiContexts) {

    let image : Handle<Image> = asset_server.load("background_main_menu.png");
    let id = egui_context.add_image(image.clone());
    commands.insert_resource(BackgroundImage(id, image));
}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<SceneType>();

        app.add_systems(Update, main_menu.run_if(in_state(SceneType::MainMenu)));
        app.add_systems(Startup, setup_main_menu);
    }
}