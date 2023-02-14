
use std::net::*;
use std::str::FromStr;
use std::sync::mpsc::*;
use rand::Rng;

use serde::*;

pub trait ByteChannel {
    fn send(&mut self, data : &[u8]) -> std::io::Result<usize>;
    fn recv(&mut self, buf : &mut [u8]) -> std::io::Result<usize>;
    fn send_to<A : ToSocketAddrs>(&mut self, data : &[u8], a : A) -> std::io::Result<usize>;
    fn recv_from(&mut self, buf : &mut [u8]) -> std::io::Result<(usize, SocketAddr)>;
}

pub trait PeerByteChannel {
    fn send_to<A : ToSocketAddrs>(&mut self, data : &[u8], a : A) -> std::io::Result<usize>;
    fn recv_from(&mut self, buf : &mut [u8]) -> std::io::Result<(usize, SocketAddr)>;
}

pub struct UpdChannel {
    pub socket : UdpSocket,
    pub bind_address : SocketAddr
}

impl ByteChannel for UpdChannel {
    fn send(&mut self, data : &[u8]) -> std::io::Result<usize> {
        self.socket.send(data)
    }

    fn recv(&mut self, buf : &mut [u8]) -> std::io::Result<usize> {
        self.socket.recv(buf)
    }

    fn send_to<A : ToSocketAddrs>(&mut self, data : &[u8], a : A) -> std::io::Result<usize> {
        self.socket.send_to(data, a)
    }

    fn recv_from(&mut self, buf : &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }
}

impl PeerByteChannel for UpdChannel {
    fn send_to<A : ToSocketAddrs>(&mut self, data : &[u8], addr : A) -> std::io::Result<usize> {
        self.socket.send_to(data, addr)
    }

    fn recv_from(&mut self, buf : &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }
}

#[derive(Default)]
pub struct EmulatedByteChannel {
    pub data : Vec<Vec<u8>>
}

impl EmulatedByteChannel {
    pub fn new() -> Self {
        Self {
            data : vec![]
        }
    }
}

pub enum EmulatedError {
    NoData
}

impl ByteChannel for EmulatedByteChannel {
    fn send(&mut self, data : &[u8]) -> std::io::Result<usize> {
        self.data.push(data.to_vec());
        Ok(data.len())
    }

    fn recv(&mut self, buf : &mut [u8]) -> std::io::Result<usize> {
        if self.data.len() > 0 {
            let data = self.data.remove(0);
            let size = data.len().min(buf.len());
            for i in 0..size {
                buf[i] = data[i];
            }
            Ok(size)
        } else {
            Ok(0)
        }
    }

    fn send_to<A : ToSocketAddrs>(&mut self, data : &[u8], a : A) -> std::io::Result<usize> {
        self.send(data)
    }

    fn recv_from(&mut self, buf : &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        if let Ok(size) = self.recv(buf) {
            Ok((size, SocketAddr::from_str("0.0.0.0:0").unwrap()))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No packages"))
        }
    }
}

pub struct EmulatedNetworkChannel {
    pub recv : Receiver<Vec<u8>>,
    pub send : Sender<Vec<u8>>,
    pub drop_rate : f32
}

impl Default for EmulatedNetworkChannel {
    fn default() -> Self {
        let (send, recv) = std::sync::mpsc::channel();
        Self { 
            recv, 
            send,
            drop_rate : 0.5
        }
    }
}

impl ByteChannel for EmulatedNetworkChannel {
    fn send(&mut self, data : &[u8]) -> std::io::Result<usize> {
        self.send.send(data.to_vec());
        Ok(data.len())
    }

    fn recv(&mut self, buf : &mut [u8]) -> std::io::Result<usize> {
        if let Ok(data) = self.recv.try_recv() {

            //check frop rate
            let mut rnd = rand::thread_rng();
            let drop_roll = rnd.gen_range(0.0..=1.0);
            if drop_roll <= self.drop_rate {
                println!("Dropped package!");
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No package"));
            }

            let size = buf.len().min(data.len());
            for idx in 0..size {
                buf[idx] = data[idx];
            }
            Ok(size)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No package"))
        }
    }

    fn send_to<A : ToSocketAddrs>(&mut self, data : &[u8], a : A) -> std::io::Result<usize> {
        self.send(data)
    }

    fn recv_from(&mut self, buf : &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        if let Ok(size) = self.recv(buf) {
            Ok((size, SocketAddr::from_str("0.0.0.0:0").unwrap()))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No packages"))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn emulated_channel() {
        let mut ch = EmulatedByteChannel::new();

        let data = [0u8, 1, 2];
        ch.send(&data).unwrap();

        let mut buf = vec![0u8; 3];
        let size = ch.recv(&mut buf).unwrap();

        assert_eq!(size, 3);
        assert_eq!(buf[0], data[0]);
        assert_eq!(buf[1], data[1]);
        assert_eq!(buf[2], data[2]);
    }
}