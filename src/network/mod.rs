use std::{net::SocketAddr, str::FromStr};

use bevy::{prelude::*, utils::Instant};
use bevy_rapier3d::rapier::crossbeam::channel::{Receiver, Sender};
use iyes_loopless::prelude::ConditionSet;

use laminar::*;

use self::protocol::ConnectionServer;

pub mod message;
pub mod channel;
pub mod protocol;
pub mod packet_socket;


pub struct NetworkPlugin;

#[derive(Resource)]
pub struct NetworkServer {
    pub server : ConnectionServer
}


impl Default for NetworkServer {
    fn default() -> Self {
        let server = ConnectionServer::new(SocketAddr::from_str("127.0.0.1:1996").unwrap(), Instant::now());

        Self {
            server
        }
    }
}

#[derive(Resource)]
pub struct NetworkClient {
    pub server : ConnectionServer,
    pub server_addr : SocketAddr
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
    client.server.manual_poll(Instant::now());
}

fn update_server(
    mut server : ResMut<NetworkServer>
) {
    server.server.manual_poll(Instant::now());
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
                    
                    let mut server = ConnectionServer::new(SocketAddr::from_str("127.0.0.1:1997").unwrap(), Instant::now());
                    server.connect_to(socket_addr.clone());
                    cmds.insert_resource(NetworkClient {
                        server,
                        server_addr : socket_addr
                    });
                }
            },
        }
    }
}