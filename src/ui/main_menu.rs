use egui::*;
use space_game::{Game, GameCommands, GuiPlugin};
use crate::scenes::setup_station_build_scene;

pub struct MainMenu {

}

impl GuiPlugin for MainMenu {
    fn shot_top_panel(&mut self, game: &mut Game, ui: &mut Ui) -> Vec<GameCommands> {
        vec![]
    }

    fn show_ui(&mut self, game: &mut Game, ctx: Context) -> Vec<GameCommands> {
        let mut cmds = vec![];
        egui::Window::new("Space sandbox")
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .show(&ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.button("New station").clicked() {
                        let cmd = GameCommands::AbstractChange(Box::new(
                           |game| {
                               setup_station_build_scene(game);
                           }
                        ));
                        cmds.push(cmd);
                    }
                    ui.button("Load station");
                    ui.button("Connect to server");
                    if ui.button("Exit").clicked() {
                        cmds.push(GameCommands::Exit);
                    }
                });
        });

        cmds
    }
}