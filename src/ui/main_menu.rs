use bevy::app::Plugin;
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::Align2;
use space_game::{Game, GameCommands, GuiPlugin, SchedulePlugin, GlobalStageStep, GameScene, SceneType};
use space_core::{ecs::*, app::App};

fn main_menu(
    mut egui_context: ResMut<EguiContext>,
    mut scene : ResMut<State<SceneType>>
) {
    egui::Window::new("Space sandbox")
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("New station").clicked() {
                    scene.set(SceneType::StationBuilding);
                }
                ui.button("Load station");
                ui.button("Connect to server");
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
        app.add_state(SceneType::MainMenu);
        app.add_system_set(
            SystemSet::on_update(SceneType::MainMenu)
            .with_system(main_menu));
    }
}