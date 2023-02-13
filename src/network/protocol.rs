use bevy::utils::HashMap;
use serde::{Serialize, Deserialize};
use bincode::*;

use super::channel::{ByteChannel, EmulatedByteChannel};

#[derive(Serialize, Deserialize)]
pub struct ChannelMsg {
    pub channel : u16,
    pub data : Vec<u8>
}


pub struct PeerRuntime<T : ByteChannel> {
    pub transport : T,
    pub in_data : HashMap<u16, Vec<Vec<u8>>>,
    pub out_data : HashMap<u16, Vec<Vec<u8>>>,
    pub buffer_size : usize
}

impl<T : ByteChannel> PeerRuntime<T> {

    fn new(transport : T, buffer_size : usize) -> PeerRuntime<T> {
        Self {
            transport,
            in_data: HashMap::new(),
            out_data: HashMap::new(),
            buffer_size,
        }
    }

    fn get_or_create_in_data(&mut self, idx : u16) -> &mut Vec<Vec<u8>> {
        if self.in_data.contains_key(&idx) == false {
            self.in_data.insert(idx.clone(), vec![]);
        }
        self.in_data.get_mut(&idx).unwrap()
    }

    fn get_or_create_out_data(&mut self, idx : u16) -> &mut Vec<Vec<u8>> {
        if self.out_data.contains_key(&idx) == false {
            self.out_data.insert(idx.clone(), vec![]);
        }
        self.out_data.get_mut(&idx).unwrap()
    }

    pub fn flush(&mut self) {
        //recv
        let mut buffer = vec![0u8; self.buffer_size];
        while let Ok(data_size) = self.transport.recv(&mut buffer) {
            if data_size == 0 {
                break;
            }
            let msg : ChannelMsg = bincode::deserialize(&buffer[0..data_size]).unwrap();
            let ch = self.get_or_create_in_data(msg.channel);
            ch.push(msg.data);
        }

        //send
        for (idx, ch) in &mut self.out_data {
            for msg in ch.iter_mut() {
                let ch_msg = ChannelMsg {
                    channel: *idx,
                    data: msg.clone(),
                };
                let ser_msg = bincode::serialize(&ch_msg).unwrap();
                self.transport.send(&ser_msg).unwrap();
            }
            ch.clear();
        }
    }

    pub fn send_msg(&mut self, ch : u16, msg : Vec<u8>) {
        let channel = self.get_or_create_out_data(ch);
        channel.push(msg);
    }

    pub fn recv_msg(&mut self, ch : u16) -> Option<Vec<u8>> {
        let channel = self.get_or_create_in_data(ch);
        if channel.len() > 0 {
            let res = channel.remove(0);
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_msg_test() {
        let msg = ChannelMsg {
            channel : 10,
            data : "Hello".as_bytes().to_vec()
        };

        let msg_bytes = bincode::serialize(&msg).unwrap();

        let msg_deser : ChannelMsg = bincode::deserialize(&msg_bytes).unwrap();

        assert_eq!(msg.channel, msg_deser.channel);
        assert_eq!(msg.data, msg_deser.data);
    }

    #[test]
    fn channel_test() {
        let data = "Hello".as_bytes().to_vec();

        let mut runtime = PeerRuntime::new(
            EmulatedByteChannel::default(),
            512
        );

        runtime.send_msg(10, data.clone());
        runtime.flush();
        runtime.flush();

        let msg = runtime.recv_msg(10);

        assert_eq!(msg, Some(data));

    }
}