use std::{net::SocketAddr, time::Duration};

use bevy::utils::{HashMap, Instant};
use bevy_rapier3d::rapier::crossbeam::channel::{Sender, Receiver};
use laminar::{Socket, Packet, SocketEvent, OrderingGuarantee, DeliveryGuarantee};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum ConnectionMsg {
    Data(Vec<u8>),
    RequestConnect,
    ApplyConnect,
    RequestHeartbit,
    Heartbit
}

#[derive(Debug)]
pub struct ConPacket {
    pub addr : SocketAddr,
    pub data : Vec<u8>,
    pub ordered : OrderingGuarantee,
    pub deliver : DeliveryGuarantee
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Data(ConPacket),
    NewClient(SocketAddr)
}

pub struct ConnectionServer {
    connections : HashMap<SocketAddr, Connection>,
    socket : Socket,
    sender :  Sender<Packet>,
    receiver : Receiver<SocketEvent>,
    time : Instant
}

impl ConnectionServer {
    pub fn new(
        addr : SocketAddr,
        time : Instant
    ) -> Self {
        let socket = Socket::bind(addr).unwrap();
        let receiver = socket.get_event_receiver();
        let sender = socket.get_packet_sender();
        Self {
            socket,
            receiver,
            sender,
            connections : HashMap::new(),
            time
        }
    }

    pub fn client_count(&self) -> usize {
        self.connections.len()
    } 

    pub fn new_client(time : Instant) -> Self {
        let socket = Socket::bind_any().unwrap();
        let receiver = socket.get_event_receiver();
        let sender = socket.get_packet_sender();
        Self {
            socket,
            receiver,
            sender,
            connections : HashMap::new(),
            time
        }
    }

    pub fn manual_poll(&mut self, time : Instant) {
        self.socket.manual_poll(time);
        self.time = time;

        let mut req_heart_addr = vec![];
        for (addr, con) in &mut self.connections {
            if (time - con.last_recv) > Duration::from_millis(100) && (time - con.last_heartbit) > Duration::from_millis(100) {
                req_heart_addr.push(addr.clone());
                con.last_heartbit = time;
            }
        }

        for addr in req_heart_addr {
            self.send_unreliable(addr, ConnectionMsg::RequestHeartbit);
        }
    }

    pub fn recv(&mut self) -> Option<ConnectionEvent> {
        if let Ok(event) = self.receiver.try_recv() {
            match &event {
                SocketEvent::Packet(packet) => {
                    if let Some(value) = self.process_packet(packet) {
                        return value;
                    }
                },
                SocketEvent::Connect(addr) => {
                    self.connections.insert(addr.clone(), Connection {
                        last_recv : self.time,
                        last_heartbit : self.time
                    });
                },
                SocketEvent::Timeout(_) => {},
                SocketEvent::Disconnect(addr) => {
                    self.connections.remove(addr);
                },
            };
        }
        None
    }

    fn process_packet(&mut self, packet: &Packet) -> Option<Option<ConnectionEvent>> {
        if let Some(con) = self.connections.get_mut(&packet.addr()) {
            con.last_recv = self.time;
        }
        let msg : ConnectionMsg = bincode::deserialize(packet.payload()).unwrap();

        match msg {
            ConnectionMsg::Data(data) => {
                let packet = ConPacket {
                    addr: packet.addr(),
                    data,
                    ordered : packet.order_guarantee(),
                    deliver : packet.delivery_guarantee()
                };
                return Some(Some(ConnectionEvent::Data(packet)));
            },
            ConnectionMsg::RequestConnect => {
                self.sender.send(
                    Packet::reliable_unordered(
                        packet.addr(), 
                bincode::serialize(&ConnectionMsg::ApplyConnect).unwrap())
                ).unwrap();
            },
            ConnectionMsg::ApplyConnect => {

            },
            ConnectionMsg::RequestHeartbit => {
                self.send_unreliable(packet.addr(), ConnectionMsg::Heartbit);
            },
            ConnectionMsg::Heartbit => {
                if let Some(con) = self.connections.get_mut(&packet.addr()) {
                    con.last_heartbit = self.time;
                }
            },
        }
        None
    }

    pub fn connect_to(&self, addr : SocketAddr) {
        self.send_reliable_unordered(addr, 
            ConnectionMsg::RequestConnect);
    }

    pub fn send_unrealiable_broadcast(&self, data : Vec<u8>) {
        for (addr, con) in &self.connections {
            self.send_unreliable(addr.clone(), ConnectionMsg::Data(data.clone()));
        }
    }

    pub fn send_realiable_broadcast(&self, data : Vec<u8>) {
        for (addr, con) in &self.connections {
            self.send_reliable_unordered(addr.clone(), ConnectionMsg::Data(data.clone()));
        }
    }

    pub fn send_reliable_unordered(&self, addr : SocketAddr, msg : ConnectionMsg) {
        let bin_msg = bincode::serialize(&msg).unwrap();
        let packet = Packet::reliable_unordered(addr, bin_msg);
        self.sender.send(packet).unwrap();
    }

    pub fn send_unreliable(&self, addr : SocketAddr, msg : ConnectionMsg) {
        let bin_msg = bincode::serialize(&msg).unwrap();
        let packet = Packet::unreliable(addr, bin_msg);
        self.sender.send(packet).unwrap();
    }
}

pub struct Connection {
    pub last_recv : Instant,
    pub last_heartbit : Instant
}