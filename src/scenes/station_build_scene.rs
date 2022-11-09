use egui::{Context, Ui};
use space_game::{Game, GameCommands, GuiPlugin};
use space_render::add_game_render_plugins;

pub fn setup_station_build_scene(game : &mut Game) {
    game.clear_plugins();
    add_game_render_plugins(game);
    game.add_gui_plugin(StationBuildMenu{});
    game.update_scene_scheldue();
}

pub struct StationBuildMenu {}

impl GuiPlugin for StationBuildMenu {
    fn show_ui(&mut self, game: &mut Game, ctx: Context) -> Vec<GameCommands> {
        let mut cmds = vec![];

        egui::SidePanel::left("Build panel").show(&ctx, |ui| {

        });

        cmds
    }
}