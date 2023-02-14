use std::{net::SocketAddr, str::FromStr};

use bevy::{prelude::*, utils::Instant};
use bevy_rapier3d::rapier::crossbeam::channel::{Receiver, Sender};
use iyes_loopless::prelude::ConditionSet;

use laminar::*;
use self::protocol::{ChannelSocket, ChannelConfig};

pub mod message;
pub mod channel;
pub mod protocol;
pub mod packet_socket;


pub struct NetworkPlugin;

#[derive(Resource)]
pub struct NetworkServer {
    pub socket : Socket,
    pub receiver : Receiver<SocketEvent>,
    pub sender : Sender<Packet>
}


impl Default for NetworkServer {
    fn default() -> Self {
        let socket = Socket::bind("127.0.0.1:1996").unwrap();
        let receiver = socket.get_event_receiver();
        let sender = socket.get_packet_sender();
        Self {
            socket,
            receiver,
            sender
        }
    }
}

#[derive(Resource)]
pub struct NetworkClient {
    pub socket : Socket,
    pub receiver : Receiver<SocketEvent>,
    pub sender : Sender<Packet>,
    pub server : SocketAddr
}

pub enum ServerNetworkCmd {
    StartServer,
    ConnectToServer(String)
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerNetworkCmd>();

        app.add_system(listen_server_cmds);

        app.add_system_set(
            ConditionSet::new()
                .run_if_resource_exists::<NetworkServer>()
                .with_system(update_server)
                .into()
        );

        app.add_system_set(
            ConditionSet::new()
                .run_if_resource_exists::<NetworkClient>()
                .with_system(update_client)
                .into()
        );
    }
}

fn update_client(
    mut client : ResMut<NetworkClient>
) {
    client.socket.manual_poll(Instant::now());
}

fn update_server(
    mut server : ResMut<NetworkServer>
) {
    server.socket.manual_poll(Instant::now());
}


fn listen_server_cmds(
    mut cmds : Commands,
    mut events : EventReader<ServerNetworkCmd>
) {
    for event in events.iter() {
        match event {
            ServerNetworkCmd::StartServer => {
                cmds.insert_resource(NetworkServer::default());
            },
            ServerNetworkCmd::ConnectToServer(addr) => {
                if let Ok(socket_addr) = SocketAddr::from_str(addr) {
                    let socket = Socket::bind("0.0.0.0:1997").unwrap();
                    let receiver = socket.get_event_receiver();
                    let sender = socket.get_packet_sender();
                    cmds.insert_resource(NetworkClient {
                        socket,
                        receiver,
                        sender,
                        server : socket_addr
                    });
                }
            },
        }
    }
}