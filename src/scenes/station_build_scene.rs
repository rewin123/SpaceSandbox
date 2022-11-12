use egui::{Context, Ui};
use space_game::{Game, GameCommands, SchedulePlugin, GlobalStageStep, EguiContext};
use space_render::add_game_render_plugins;
use space_core::{ecs::*, app::App};

pub fn setup_station_build_scene(game : &mut Game) {
    game.clear_plugins();
    add_game_render_plugins(game);
    game.add_schedule_plugin(StationBuildMenu{});
    game.update_scene_scheldue();
}

fn station_menu(
    ctx : Res<EguiContext>
) {
    egui::SidePanel::left("Build panel").show(&ctx, |ui| {
        egui::Grid::new("Floor block grid").show(ui, |ui| {

        });
    });
}

pub struct StationBuildMenu {}

impl SchedulePlugin for StationBuildMenu {
    fn get_name(&self) -> space_game::PluginName {
        space_game::PluginName::Text("Station build menu".into())
    }

    fn add_system(&self, app : &mut App) {
        app.add_system_to_stage(GlobalStageStep::Render, station_menu);
    }
    // fn show_ui(&mut self, game: &mut Game, ctx: Context) -> Vec<GameCommands> {
    //     let mut cmds = vec![];

    //     egui::SidePanel::left("Build panel").show(&ctx, |ui| {
    //         egui::Grid::new("Floor block grid").show(ui, |ui| {

    //         });
    //     });

    //     cmds
    // }
}