use egui::{Context, Ui};
use space_game::{Game, GameCommands, SchedulePlugin, GlobalStageStep, EguiContext, SceneType};
use space_render::add_game_render_plugins;
use space_core::{ecs::*, app::App};

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
        app.add_system_set(
            SystemSet::on_update(SceneType::StationBuilding)
                .with_system(station_menu));

    }
}