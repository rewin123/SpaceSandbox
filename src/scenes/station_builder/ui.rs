use bevy::prelude::*;
use bevy_egui::*;
use bevy_rapier3d::rapier::control::DynamicRayCastVehicleController;

use crate::ship::common::AllVoxelInstances;

use super::*;

pub struct StationBuilderUI;

#[derive(Resource, Default)]
pub struct CachedSavedShips {
    pub paths : Vec<String>
}

impl Plugin for StationBuilderUI {
    fn build(&self, app: &mut App) {
        app.insert_resource(CachedSavedShips::default());
    }
}

pub fn ship_build_menu(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    voxel_instances : Res<AllVoxelInstances>,
    mut ctx : ResMut<EguiContext>,
    mut block : ResMut<StationBuildBlock>,
    mut active_windows : ResMut<ActiveWindows>,
    mut cahed_saved_paths : ResMut<CachedSavedShips>
) {
    egui::SidePanel::left("Build panel").show(ctx.ctx_mut(), |ui| {

        if ui.button("Clear level").clicked() {
            block.cmd = StationBuildCmds::ClearAll;
        }
        ui.separator();
        if ui.button("Quick load").clicked() {
            block.cmd = StationBuildCmds::QuickLoad;
        }
        if ui.button("Quick save").clicked() {
            block.cmd = StationBuildCmds::QuickSave;
        }
        ui.separator();

        if ui.button("Load from file").clicked() {
            active_windows.load_ship = !active_windows.load_ship;

            let mut paths = vec![];
            for entry in std::fs::read_dir("saves").unwrap() {
                if let Ok(file) = entry {
                    if let Ok(file_tp) = file.file_type() {
                        if file_tp.is_file() {
                            if file.path().to_str().unwrap().contains("scn.ron") {
                                paths.push(file.path().to_str().unwrap().to_string());
                            }
                        }
                    }
                }
            }

            cahed_saved_paths.paths = paths;
        }

        ui.separator();
        for inst in &voxel_instances.configs {
            if ui.button(&inst.name).clicked() {

                let e = inst.create.build(&mut cmds, &asset_server);
                cmds.entity(e).insert(ActiveBlock);

                if let Some(prev_e) = block.e {
                    cmds.entity(prev_e).despawn_recursive();
                }

                block.e = Some(e);
                block.instance = Some(inst.instance.clone());
                block.cur_name = inst.name.clone();
            }
        }
    });
}

pub fn load_ship_ui(
    mut ctx : ResMut<EguiContext>,
    mut active_windows : ResMut<ActiveWindows>,
    saved_ships : Res<CachedSavedShips>,
    mut load_ship_cmd : EventWriter<CmdShipLoad>) {
        egui::Window::new("Select ship to load")
            .show(ctx.ctx_mut(), |ui| {

            ui.label("Ships:");
            for path in &saved_ships.paths {
                if ui.button(path).clicked() {
                    load_ship_cmd.send(CmdShipLoad(path.clone()));
                    active_windows.load_ship = false;
                }
            }

            ui.separator();

            if ui.button("Close").clicked() {
                active_windows.load_ship = false;
            }
        });
}