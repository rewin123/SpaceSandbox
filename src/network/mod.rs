use std::{net::SocketAddr, str::FromStr};

use bevy::{prelude::*, utils::{Instant, HashMap}, reflect::erased_serde::Serialize};
use bevy_rapier3d::rapier::crossbeam::channel::{Receiver, Sender};
use iyes_loopless::prelude::ConditionSet;

use laminar::*;
use serde::de::DeserializeOwned;

use self::{protocol::ConnectionServer, packet_socket::SendDestination};

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

pub struct MessageChannel<T> {
    pub sender : Sender<(SendDestination, T)>,
    pub receiver : Receiver<(SocketAddr, T)>
}

pub type ChannelID = u16;



pub trait ByteTransform {
    fn from_net(&self, data : Vec<u8>, addr : SocketAddr);
    fn to_net(&self) -> Option<(SendDestination, Vec<u8>)>;
    fn deliver(&self) -> DeliveryGuarantee;
}

pub struct SimpleByteTransfer<T : Serialize + DeserializeOwned> {
    pub from_net : Sender<(SocketAddr, T)>,
    pub to_net : Receiver<(SendDestination, T)>
}

impl<T : serde::Serialize + serde::de::DeserializeOwned> ByteTransform for SimpleByteTransfer<T> {
    fn from_net(&self, data : Vec<u8>, addr : SocketAddr) {
        let msg = bincode::deserialize(&data).unwrap();
        self.from_net.send((addr, msg)).unwrap();
    }

    fn to_net(&self) -> Option<(SendDestination, Vec<u8>)> {
        if let Ok((dst, msg)) = self.to_net.try_recv() {
            let data = bincode::serialize(&msg).unwrap();
            Some((dst, data))
        } else {
            None
        }
    }

    fn deliver(&self) -> DeliveryGuarantee {
        DeliveryGuarantee::Reliable
    }
}

#[derive(Resource, Default)]
pub struct NetworkSplitter {
    pub splits : HashMap<ChannelID, Box<dyn ByteTransform + Send + Sync>>,
    pub indexer : ChannelID
}

impl NetworkSplitter {
    pub fn register_type<T : serde::Serialize + DeserializeOwned + Send + Sync + 'static>(&mut self) -> MessageChannel<T> {
        let id = self.indexer;
        self.indexer += 1;

        let (to_net_send, to_net_recv) = bevy_rapier3d::rapier::crossbeam::channel::unbounded();
        let (from_net_send, from_net_recv) = bevy_rapier3d::rapier::crossbeam::channel::unbounded();

        let msg = MessageChannel::<T> {
            sender : to_net_send,
            receiver : from_net_recv
        };

        let simple_transform = SimpleByteTransfer::<T> {
            from_net: from_net_send,
            to_net: to_net_recv,
        };

        self.splits.insert(id, Box::new(simple_transform));

        msg
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ServerNetworkCmd>();

        app.add_system(listen_server_cmds);
        app.insert_resource(NetworkSplitter::default());

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
    mut client : ResMut<NetworkClient>,
    mut splitter : ResMut<NetworkSplitter>
) {
    client.server.manual_poll(Instant::now());

    //recv
    while let Some(msg) = client.server.recv() {
        match msg {
            protocol::ConnectionEvent::Data(data) => {
                let (id, raw_data) : (ChannelID, Vec<u8>) = bincode::deserialize(&data.data).unwrap();
                if let Some(ch) = splitter.splits.get_mut(&id) {
                    ch.from_net(raw_data, data.addr);
                }
            },
        }
    }

    //send
    for (id, ch) in &splitter.splits {
        while let Some((dst, raw_data)) = ch.to_net() {
            let data = bincode::serialize(&(*id, raw_data)).unwrap();
            match dst {
                SendDestination::Target(addr) => client.server.send_reliable_unordered(addr, protocol::ConnectionMsg::Data(data)),
                SendDestination::Broadcast => client.server.send_realiable_broadcast(data),
            }
        }
    }
}

fn update_server(
    mut server : ResMut<NetworkServer>,
    mut splitter : ResMut<NetworkSplitter>
) {
    server.server.manual_poll(Instant::now());

    //recv
    while let Some(msg) = server.server.recv() {
        match msg {
            protocol::ConnectionEvent::Data(data) => {
                let (id, raw_data) : (ChannelID, Vec<u8>) = bincode::deserialize(&data.data).unwrap();
                if let Some(ch) = splitter.splits.get_mut(&id) {
                    ch.from_net(raw_data, data.addr);
                }
            },
        }
    }

    //send
    for (id, ch) in &splitter.splits {
        while let Some((dst, raw_data)) = ch.to_net() {
            let data = bincode::serialize(&(*id, raw_data)).unwrap();
            match dst {
                SendDestination::Target(addr) => server.server.send_reliable_unordered(addr, protocol::ConnectionMsg::Data(data)),
                SendDestination::Broadcast => server.server.send_realiable_broadcast(data),
            }
        }
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