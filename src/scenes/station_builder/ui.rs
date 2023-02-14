use bevy::prelude::*;
use bevy_egui::*;
use laminar::Packet;

use crate::{ship::common::AllVoxelInstances, network::{NetworkServer, NetworkClient, ServerNetworkCmd, packet_socket::{SendPacket, SendDestination}, protocol::{ConnectionEvent, ConnectionMsg}, MessageChannel, NetworkSplitter}};

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

        app.add_startup_system(setup_chat);
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
    mut ctx : ResMut<EguiContext>,
    mut block : ResMut<StationBuildBlock>,
    mut active_windows : ResMut<ActiveWindows>,
    mut cahed_saved_paths : ResMut<CachedSavedShips>,
    mut cmd_save : EventWriter<CmdShipSave>,
    mut state : ResMut<BuildMenuState>,
    mut server_op : Option<ResMut<NetworkServer>>,
    mut client_op : Option<ResMut<NetworkClient>>,
    mut network_cmds : EventWriter<ServerNetworkCmd>,
    mut chat_channel : ResMut<NetworkChat>
) {
    egui::SidePanel::left("Build panel").show(ctx.ctx_mut(), |ui| {
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
            } else {
                if ui.button("Start server").clicked() {
                    network_cmds.send(ServerNetworkCmd::StartServer);
                }
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

        ui.add(egui::TextEdit::singleline(&mut state.save_name));
        if ui.button("Save by name").clicked() {
            cmd_save.send(CmdShipSave(block.ship, format!("saves/{}.scn.ron", state.save_name)));
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