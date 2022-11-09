use egui::*;
use space_game::{Game, GameCommands, GuiPlugin};

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
                    ui.button("New station");
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