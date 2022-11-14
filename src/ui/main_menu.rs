use egui::*;
use space_game::{Game, GameCommands, GuiPlugin, SchedulePlugin, GlobalStageStep, EguiContext, GameScene, SceneType};
use space_core::{ecs::*, app::App};

fn main_menu(
    ctx : Res<EguiContext>,
    mut scene : ResMut<State<SceneType>>
) {
    egui::Window::new("Space sandbox")
        .resizable(false)
        .collapsible(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("New station").clicked() {
                    scene.set(SceneType::StationBuilding).unwrap();
                }
                ui.button("Load station");
                ui.button("Connect to server");
                if ui.button("Exit").clicked() {
                    // cmds.push(GameCommands::Exit);
                }
            });
    });
}

pub struct MainMenu {

}

impl SchedulePlugin for MainMenu {
    fn get_name(&self) -> space_game::PluginName {
        space_game::PluginName::Text("Main menu".into())
    }

    fn add_system(&self, app : &mut App) {
        app.add_system_set(
            SystemSet::on_update(SceneType::MainMenu)
                .with_system(main_menu));
    }
}