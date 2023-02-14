use std::{net::SocketAddr, str::FromStr};

use bevy::prelude::*;
use iyes_loopless::prelude::ConditionSet;

use self::protocol::{ChannelSocket, ChannelConfig};

pub mod message;
pub mod channel;
pub mod protocol;
pub mod packet_socket;

pub struct NetworkPlugin;

#[derive(Resource)]
pub struct NetworkServer {
    pub socket : ChannelSocket
}

fn prepare_socket_channels(mut socket : ChannelSocket) -> ChannelSocket {

    socket.register_channel(ChannelConfig {
        tp: protocol::ChannelType::Unrealiable,
        id: 0,
    });

    socket
}

impl Default for NetworkServer {
    fn default() -> Self {
        let addr = SocketAddr::from_str("0.0.0.0:1996").unwrap();
        let socket = ChannelSocket::new(addr);
        Self {
            socket : prepare_socket_channels(socket)
        }
    }
}

#[derive(Resource)]
pub struct NetworkClient {
    pub socket : ChannelSocket,
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
    client.socket.update();
}

fn update_server(
    mut server : ResMut<NetworkServer>
) {
    server.socket.update();
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
                    let socket = prepare_socket_channels(ChannelSocket::new(SocketAddr::from_str("0.0.0.0:1997").unwrap()));
                    socket.socket.connect(socket_addr.clone());
                    cmds.insert_resource(NetworkClient {
                        socket: socket,
                        server : socket_addr
                    });
                }
            },
        }
    }
}