use std::{net::{UdpSocket, SocketAddr}, time::Duration};

use bevy::{prelude::*, utils::{HashMap, HashSet, Instant}};
use serde::*;

use super::packet_socket::{SendPacket, RecvPacket, PacketSocket, SendDestination};

pub type ChannelID = u16;

pub trait InnerChannel {
    fn from_net(&mut self, packet : RecvPacket);
    fn to_net(&mut self) -> Option<SendPacket>;

    fn recv(&mut self) -> Option<RecvPacket>;
    fn send(&mut self, msg : SendPacket);

    fn update(&mut self);
}

#[derive(Serialize, Deserialize)]
pub struct ChannelMsg {
    pub ch : ChannelID,
    pub data : Vec<u8>
}

pub struct ChannelSocket {
    pub socket : PacketSocket,
    pub channels : HashMap<ChannelID, Box<dyn InnerChannel + Sync + Send>>
}

pub enum ChannelType {
    Unrealiable,
    Realiable
}

pub struct ChannelConfig {
    pub tp : ChannelType,
    pub id : ChannelID
}

impl ChannelSocket {

    pub fn new(addr : SocketAddr) -> Self {
        let socket = PacketSocket::new(addr);

        Self {
            socket,
            channels : HashMap::new()
        }
    }

    pub fn update(&mut self) {
        self.socket.update();
        //recv
        while let Some(packet) = self.socket.recv() {
            let ch_msg : ChannelMsg = bincode::deserialize(&packet.data).unwrap();
            if let Some(ch) = self.channels.get_mut(&ch_msg.ch) {
                ch.from_net(RecvPacket {
                    client: packet.client,
                    data: ch_msg.data,
                });
            }
        }

        for (ch_idx, ch) in &mut self.channels {
            ch.update();

            while let Some(msg) = ch.to_net() {
                let ch_msg = ChannelMsg {
                    ch: *ch_idx,
                    data: msg.data,
                };
                let send_msg = SendPacket {
                    dst : msg.dst,
                    data : bincode::serialize(&ch_msg).unwrap()
                };
                self.socket.send(send_msg);
            }
        }
    }

    pub fn register_channel(&mut self, config : ChannelConfig) {
        match config.tp {
            ChannelType::Unrealiable => {
                self.channels.insert(config.id, Box::new(UnrealibaleChannel::default()));
            },
            ChannelType::Realiable => {
                self.channels.insert(config.id, Box::new(ReliableChannel::default()));
            },
        }
    }

    pub fn recv_channel(&mut self, id : ChannelID) -> Option<RecvPacket> {
        if let Some(ch) = self.channels.get_mut(&id) {
            ch.recv()
        } else {
            None
        }
    }

    pub fn send_channel(&mut self, id : ChannelID, msg : SendPacket) {
        if let Some(ch) = self.channels.get_mut(&id) {
            ch.send(msg);
        } else {
            
        }
    }
}


#[derive(Default)]
pub struct UnrealibaleChannel {
    to_net : Vec<SendPacket>,
    from_net : Vec<RecvPacket>
}

impl InnerChannel for UnrealibaleChannel {
    fn from_net(&mut self, packet : RecvPacket) {
        self.from_net.push(packet);
    }

    fn to_net(&mut self) -> Option<SendPacket> {
        if self.to_net.len() > 0 {
            Some(self.to_net.remove(0))
        } else {
            None
        }
    }

    fn recv(&mut self) -> Option<RecvPacket> {
        if self.from_net.len() > 0 {
            Some(self.from_net.remove(0))
        } else {
            None
        }
    }

    fn send(&mut self, msg : SendPacket) {
        self.to_net.push(msg);
    }

    fn update(&mut self) {
        
    }
}

type MsgID = u32;

#[derive(Serialize, Deserialize)]
pub enum ReliableMsg {
    Got(MsgID),
    Msg(MsgID, Vec<u8>)
}

pub struct ReliableSnapshot {
    pub start : Instant,
    pub msg : SendPacket,
    pub last_try : Instant
}

pub struct ReliableChannel {
    to_net : Vec<SendPacket>,
    from_net : Vec<RecvPacket>,

    wait_accept : HashMap<MsgID, ReliableSnapshot>,
    gotted : HashMap<MsgID, Instant>,
    round_trip : Duration,
    max_round_trip : Duration,
    max_got_time : Duration,
    indexer : MsgID
}

impl Default for ReliableChannel {
    fn default() -> Self {
        Self { 
            to_net: Default::default(), 
            from_net: Default::default(), 
            wait_accept: Default::default(), 
            gotted: Default::default(), 
            round_trip: Duration::from_millis(100), 
            max_round_trip: Duration::from_millis(2000), 
            max_got_time: Duration::from_millis(10000), 
            indexer: 0
        }

    }
}

impl ReliableChannel {
    fn get_id(&mut self) -> MsgID {
        let id = self.indexer;
        self.indexer.wrapping_add(1);
        id
    }
}

impl InnerChannel for ReliableChannel {
    fn from_net(&mut self, packet : RecvPacket) {
        let rel_msg : ReliableMsg = bincode::deserialize(&packet.data).unwrap();
        match rel_msg {
            ReliableMsg::Got(id) => {
                self.wait_accept.remove(&id);
            },
            ReliableMsg::Msg(id, data) => {
                if !self.gotted.contains_key(&id) {
                    self.gotted.insert(id, Instant::now());
                    self.from_net.push(RecvPacket {
                        client: packet.client,
                        data: data,
                    });
                }
                self.to_net.push(SendPacket {
                    dst: SendDestination::Target(packet.client.clone()),
                    data : bincode::serialize(&ReliableMsg::Got(id)).unwrap()
                });
            },
        }
        
        
    }

    fn to_net(&mut self) -> Option<SendPacket> {
        if self.to_net.len() > 0 {
            Some(self.to_net.remove(0))
        } else {
            None
        }
    }

    fn recv(&mut self) -> Option<RecvPacket> {
        if self.from_net.len() > 0 {
            Some(self.from_net.remove(0))
        } else {
            None
        }
    }

    fn send(&mut self, msg : SendPacket) {
        let id = self.get_id();
        let rel_msg = ReliableMsg::Msg(id, msg.data);
        let real_msg = SendPacket {
            dst : msg.dst.clone(),
            data : bincode::serialize(&rel_msg).unwrap()
        };
        self.to_net.push(real_msg.clone());

        let snapshot = ReliableSnapshot {
            start: Instant::now(),
            msg : real_msg,
            last_try: Instant::now(),
        };
        self.wait_accept.insert(id, snapshot);
    }

    fn update(&mut self) {
        let time = Instant::now();
        { //delete long time gotted id
            let del_idx : Vec<MsgID> = self.gotted.iter().filter(|(id, start)| {
                (time - **start) > self.max_got_time
            }).map(|(id, start)| {
                    *id
            }).collect();
            for id in del_idx {
                self.gotted.remove(&id);
            }
        }

        { //resend data if not responce
            let mut del_id = vec![];
            for (id, snap) in &mut self.wait_accept {
                if (time - snap.last_try) > self.round_trip {
                    self.to_net.push(snap.msg.clone());
                    snap.last_try = time;
                }
                if (time - snap.start) > self.max_round_trip {
                    del_id.push(*id);
                }
            }

            for id in del_id {
                self.wait_accept.remove(&id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr, time::Duration};

    use crate::network::{protocol::{ChannelSocket, ChannelConfig}, packet_socket::{SendPacket, SendDestination}};


    #[test]
    fn channel_packet_test() {
        let server_addr = SocketAddr::from_str("127.0.0.1:1998").unwrap();
        let client_addr = SocketAddr::from_str("127.0.0.1:1999").unwrap();

        let mut server = ChannelSocket::new(server_addr.clone());
        let mut client = ChannelSocket::new(client_addr.clone());

        let channel_id = 1;

        server.register_channel(ChannelConfig {
            tp: crate::network::protocol::ChannelType::Unrealiable,
            id: channel_id,
        });
        client.register_channel(ChannelConfig {
            tp: crate::network::protocol::ChannelType::Unrealiable,
            id: channel_id,
        });


        client.send_channel(channel_id, SendPacket {
            dst: SendDestination::Target(server_addr.clone()),
            data: vec![0u8, 1u8, 2u8],
        });
        client.update();
        client.update();
        client.update();
        std::thread::sleep(Duration::from_millis(10));
        server.update();
        server.update();
        server.update();

        let msg = server.recv_channel(channel_id).unwrap();

        assert_eq!(msg.data[0], 0);
        assert_eq!(msg.data[1], 1);
        assert_eq!(msg.data[2], 2);
    }
}