use std::default::default;
use bevy::asset::AssetServer;
use egui::{Context, Ui};
use space_game::{Game, GameCommands, SchedulePlugin, GlobalStageStep, EguiContext, SceneType, RonAssetPlugin};
use space_render::add_game_render_plugins;
use space_core::{ecs::*, app::App};
use space_core::serde::*;
use bevy::reflect::*;
use bevy::asset::*;

#[derive(Default, Deserialize, TypeUuid)]
#[uuid = "fce6d1f5-4317-4077-b23e-6099747b08dd"]
struct BlockDesc {
    pub name : String,
    pub path : String
}


#[derive(Resource, Default)]
struct StationBlocks {
    pub panels : Vec<BlockDesc>
}


fn station_menu(
    ctx : Res<EguiContext>,
    panels : Res<StationBlocks>
) {
    egui::SidePanel::left("Build panel").show(&ctx, |ui| {
        egui::Grid::new("Floor block grid").show(ui, |ui| {

        });
    });
}

fn init_station_build(
    mut commands : Commands,
    mut assets : Res<AssetServer>
) {
    println!("Start station build system");
    commands.insert_resource(StationBlocks::default());

    let handle : Handle<BlockDesc> = assets.load("ss13/walls_configs/metal_grid.wall");
}


pub struct StationBuildMenu {}

impl SchedulePlugin for StationBuildMenu {
    fn get_name(&self) -> space_game::PluginName {
        space_game::PluginName::Text("Station build menu".into())
    }

    fn add_system(&self, app : &mut App) {

        app.add_plugin(RonAssetPlugin::<BlockDesc>{ ext: vec!["wall"], ..default() });

        app.add_system_set(SystemSet::on_enter(SceneType::StationBuilding)
            .with_system(init_station_build));

        app.add_system_set(
            SystemSet::on_update(SceneType::StationBuilding)
                .with_system(station_menu));

    }
}