use bevy::prelude::*;
use bevy_egui::*;


use crate::{ship::common::AllVoxelInstances, network::{NetworkServer, NetworkClient, ServerNetworkCmd, packet_socket::{SendDestination}, MessageChannel, NetworkSplitter}, control::Action};

use super::*;

pub struct StationBuilderUI;

#[derive(Resource, Default)]
pub struct CachedSavedShips {
    pub paths : Vec<String>
}

impl Plugin for StationBuilderUI {
    fn build(&self, app: &mut App) {
        app.insert_resource(CachedSavedShips::default());
        app.insert_resource(BuildMenuState::default());

        app.add_systems(Startup, setup_chat);
    }
}

#[derive(Resource)]
pub struct NetworkChat {
    pub channel : MessageChannel<String>
}

fn setup_chat(
    mut cmds : Commands,
    mut splitters : ResMut<NetworkSplitter>
) {
    cmds.insert_resource(NetworkChat {
        channel : splitters.register_type::<String>()
    });
}

#[derive(Resource, Default)]
pub struct BuildMenuState {
    pub save_name : String,
    pub connect_ip : String,
    pub chat : String,
    pub chat_msg : String
}

pub fn ship_build_menu(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    voxel_instances : Res<AllVoxelInstances>,
    mut ctx : Query<&mut EguiContext>,
    mut block : ResMut<StationBuildBlock>,
    mut active_windows : ResMut<ActiveWindows>,
    mut cahed_saved_paths : ResMut<CachedSavedShips>,
    mut cmd_save : EventWriter<CmdShipSave>,
    mut state : ResMut<BuildMenuState>,
    server_op : Option<ResMut<NetworkServer>>,
    client_op : Option<ResMut<NetworkClient>>,
    network_cmds : EventWriter<ServerNetworkCmd>,
    chat_channel : ResMut<NetworkChat>,
    input : ResMut<Input<Action>>
) {
    let mut ctx = ctx.single_mut();
    egui::SidePanel::left("Build panel").show(ctx.get_mut(), |ui| {
        network_chat(client_op, server_op, ui, chat_channel, &mut state, network_cmds);

        if ui.button("Play").clicked() {
            block.cmd = StationBuildCmds::GoToFPS;
        }

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
                        if file_tp.is_file() && file.path().to_str().unwrap().contains("scn.ron") {
                            paths.push(file.path().to_str().unwrap().to_string());
                        }
                    }
                }
            }

            cahed_saved_paths.paths = paths;
        }

        ui.add(egui::TextEdit::singleline(&mut state.save_name));
        if ui.button("Save by name").clicked() {
            cmd_save.send(CmdShipSave(block.ship, format!("saves/{}.scn.ron", state.save_name)));
        }

        ui.separator();

        let step = 0.25;
        match &mut block.mode {
            BuildMode::SingleOnY(lvl) => {
                if input.just_pressed(Action::Build(control::BuildAction::LevelDown)) {
                    *lvl -= step;
                }
                if input.just_pressed(Action::Build(control::BuildAction::LevelUp)) {
                    *lvl += step;
                }
                ui.add(egui::DragValue::new(lvl)
                    .prefix("Build z level:")
                    .speed(0.5)
                    .fixed_decimals(1));
            },
        }

        if block.z_slice.is_none() {
            if ui.button("Enable y slicing").clicked() {
                block.z_slice = Some(10.0);
            }
        } else if let Some(z_slice) = &mut block.z_slice {
            ui.add(egui::DragValue::new(z_slice)
                .prefix("Build z slice:")
            );
            if ui.button("Disable y slicing").clicked() {
                block.z_slice = None;
            }
        }

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

fn network_chat(mut client_op: Option<ResMut<NetworkClient>>, mut server_op: Option<ResMut<NetworkServer>>, ui: &mut egui::Ui, chat_channel: ResMut<NetworkChat>, state: &mut ResMut<BuildMenuState>, mut network_cmds: EventWriter<ServerNetworkCmd>) {
    if client_op.is_none() {
        if let Some(server) = &mut server_op {
            ui.label(format!("Clients: {}", server.server.client_count()));
            if let Ok((from, msg)) = chat_channel.channel.receiver.try_recv() {
                state.chat = format!("{}\n{}:{}", state.chat, from, msg);
            }
            ui.label(&state.chat);

            ui.add(egui::TextEdit::singleline(&mut state.chat_msg));

            if ui.button("Send message").clicked() {

                chat_channel.channel.sender.send((
                    SendDestination::Broadcast,
                    state.chat_msg.clone()
                )).unwrap();
                // server.sender.send(Packet::reliable_unordered(, payload))
                state.chat_msg = "".to_string();
            }
        } else if ui.button("Start server").clicked() {
            network_cmds.send(ServerNetworkCmd::StartServer);
        }
    }

    if server_op.is_none() {
        if let Some(client) = &mut client_op {

            ui.label(format!("Clients: {}", client.server.client_count()));

            if let Ok((from, msg)) = chat_channel.channel.receiver.try_recv() {
                state.chat = format!("{}\n{}:{}", state.chat, from, msg);
            }
            ui.label(&state.chat);

            ui.add(egui::TextEdit::singleline(&mut state.chat_msg));

            if ui.button("Send message").clicked() {

                chat_channel.channel.sender.send((
                    SendDestination::Broadcast,
                    state.chat_msg.clone()
                )).unwrap();
                // server.sender.send(Packet::reliable_unordered(, payload))
                state.chat_msg = "".to_string();
            }
        } else {
            ui.add(egui::TextEdit::singleline(&mut state.connect_ip));
            if ui.button("Connect to server").clicked() {
                network_cmds.send(ServerNetworkCmd::ConnectToServer(state.connect_ip.clone()));
            }
        }
    }
}

pub fn load_ship_ui(
    mut ctx : Query<&mut EguiContext>,
    mut active_windows : ResMut<ActiveWindows>,
    saved_ships : Res<CachedSavedShips>,
    mut load_ship_cmd : EventWriter<CmdShipLoad>) {
        let mut ctx = ctx.single_mut();
        egui::Window::new("Select ship to load")
            .show(ctx.get_mut(), |ui| {

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