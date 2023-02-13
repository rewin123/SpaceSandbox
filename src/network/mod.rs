use renet::*;
use bevy::prelude::*;

pub mod message;
pub mod channel;
pub mod protocol;

pub struct NetworkPlugin;

#[derive(Resource)]
pub struct NetworkServer {

}

impl Default for NetworkServer {
    fn default() -> Self {
        Self {

        }
    }
}

#[derive(Resource)]
pub struct NetworkClient {

}

pub enum ServerNetworkCmd {
    StartServer,
    ConnectToServer
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerNetworkCmd>();

        app.add_system(listen_server_cmds);
    }
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
            ServerNetworkCmd::ConnectToServer => {

            },
        }
    }
}