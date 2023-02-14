use std::net::*;


pub struct RecvPacket {
    pub client : SocketAddr,
    pub data : Vec<u8>
}

#[derive(Clone)]
pub enum SendDestination {
    Target(SocketAddr),
    Broadcast
}

#[derive(Clone)]
pub struct SendPacket {
    pub dst : SendDestination,
    pub data : Vec<u8>
}

pub struct PacketSocket {
    socket : UdpSocket,
    buffer_size : usize,
    from_net : Vec<RecvPacket>,
    to_net : Vec<SendPacket> 
}

impl PacketSocket {

    pub fn new(addr : SocketAddr) -> Self {

        let socket = UdpSocket::bind(addr).unwrap();
        socket.set_nonblocking(true).unwrap();
        socket.set_broadcast(true).unwrap();
        Self { 
            socket, 
            buffer_size: 16000, 
            from_net: vec![], 
            to_net: vec![] 
        }
    }

    pub fn connect(&self, addr : SocketAddr) {
        self.socket.connect(addr);
    }

    pub fn send(&mut self, packet : SendPacket) {
        self.to_net.push(packet);
    }

    pub fn recv(&mut self) -> Option<RecvPacket> {
        if self.from_net.len() > 0 {
            return Some(self.from_net.remove(0));
        }
        None
    }

    pub fn update(&mut self) {
        let mut buffer = vec![0u8; self.buffer_size];

        while let Ok((data_size, address)) = self.socket.recv_from(&mut buffer) {
            let packet = RecvPacket {
                client: address,
                data: buffer[0..data_size].to_vec(),
            };
            self.from_net.push(packet);
        }

        for packet in &self.to_net {
            match &packet.dst {
                SendDestination::Target(addr) => {
                    self.socket.send_to(&packet.data, addr);
                },
                SendDestination::Broadcast => {
                    self.socket.send(&packet.data);
                },
            }
        }
        self.to_net.clear();
    }
}

#[cfg(test)]
mod tests {
    use std::{net::SocketAddr, str::FromStr};

    use super::{PacketSocket, SendPacket};


    #[test]
    fn packet_test() {
        let server_addr = SocketAddr::from_str("127.0.0.1:1996").unwrap();
        let client_addr = SocketAddr::from_str("127.0.0.1:1997").unwrap();

        let mut server = PacketSocket::new(server_addr.clone());
        let mut client = PacketSocket::new(client_addr.clone());

        client.send(SendPacket {
            dst: super::SendDestination::Target(server_addr.clone()),
            data: vec![0u8, 1u8, 2u8],
        });
        client.update();
        server.update();

        let msg = server.recv().unwrap();

        assert_eq!(msg.data[0], 0);
        assert_eq!(msg.data[1], 1);
        assert_eq!(msg.data[2], 2);
    }
}