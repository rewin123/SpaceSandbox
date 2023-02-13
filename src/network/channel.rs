
use std::net::*;
use std::sync::mpsc::*;


use serde::*;

pub trait ByteChannel {
    fn send(&mut self, data : &[u8]) -> std::io::Result<usize>;
    fn recv(&mut self, buf : &mut [u8]) -> std::io::Result<usize>;
}

pub trait MessageChannel<T> {
    fn get_channel(&mut self) -> (&mut Receiver<T>, Sender<T>);
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
}

pub struct MessageSenderChannel<T> {
    pub in_channel : (Receiver<T>, Sender<T>),
    pub out_channel : (Receiver<T>, Sender<T>)
}

impl<T> MessageChannel<T> for MessageSenderChannel<T> {
    fn get_channel(&mut self) -> (&mut Receiver<T>, Sender<T>) {
        let rec = &mut self.in_channel.0;
        let sen = self.out_channel.1.clone();
        (rec, sen)
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