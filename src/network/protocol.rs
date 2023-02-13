use bevy::utils::HashMap;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use bincode::*;
use std::sync::mpsc::*;
use serde::*;

use super::channel::{ByteChannel, EmulatedByteChannel};

pub type ChannelID = u16;

pub trait ByteMsgs {
    fn msg_to_send(&mut self) -> Option<Vec<u8>>;
    fn recv_msg(&mut self, msg : Vec<u8>);
}

#[derive(Serialize, Deserialize)]
pub struct ChannelMsg {
    pub channel : ChannelID,
    pub data : Vec<u8>
}

pub struct MessageChannel<T> {
    pub recv : Receiver<T>,
    pub send : Sender<T>
}

pub struct MessageTransform<T : Serialize + DeserializeOwned> {
    pub to_bytes : Receiver<T>,
    pub from_bytes : Sender<T>
}

impl<T : Serialize + DeserializeOwned> ByteMsgs for MessageTransform<T> {
    fn msg_to_send(&mut self) -> Option<Vec<u8>> {
        if let Ok(data) = self.to_bytes.try_recv() {
            let msg = bincode::serialize(&data).unwrap();
            Some(msg)
        } else {
            None
        }
    }
    fn recv_msg(&mut self, msg : Vec<u8>) {
        let data = bincode::deserialize(&msg).unwrap();
        self.from_bytes.send(data);
    }
}

pub struct PeerRuntime<T : ByteChannel> {
    pub transport : T,
    pub channels : HashMap<ChannelID, Box<dyn ByteMsgs>>,
    pub buffer_size : usize,
}

impl<T : ByteChannel> PeerRuntime<T> {

    pub fn new(transport : T, buffer_size : usize) -> PeerRuntime<T> {
        Self {
            transport,
            channels : HashMap::new(),
            buffer_size,
        }
    }

    pub fn build_channel<D : Serialize + DeserializeOwned + 'static>(&mut self, ch : ChannelID) -> MessageChannel<D> {
        let (to_net_send, to_net_recv) = channel();
        let (from_net_send, from_net_recv) = channel();
        
        let msg_ch = MessageChannel::<D> {
            recv : from_net_recv,
            send : to_net_send
        };

        let transfer = MessageTransform::<D> {
            to_bytes : to_net_recv,
            from_bytes : from_net_send
        };

        if self.channels.contains_key(&ch) {
            panic!("Channel already exist!");
        }

        self.channels.insert(ch, Box::new(transfer));

        msg_ch
    }

    pub fn flush(&mut self) {
        //recv
        let mut buffer = vec![0u8; self.buffer_size];
        while let Ok(data_size) = self.transport.recv(&mut buffer) {
            if data_size == 0 {
                break;
            }
            let msg : ChannelMsg = bincode::deserialize(&buffer[0..data_size]).unwrap();
            if let Some(ch) = self.channels.get_mut(&msg.channel) {
                ch.recv_msg(msg.data);
            }
        }

        //send
        for (idx, ch) in &mut self.channels {
            while let Some(msg) = ch.msg_to_send() {
                let ch_msg = ChannelMsg {
                    channel: *idx,
                    data: msg.clone(),
                };
                let ser_msg = bincode::serialize(&ch_msg).unwrap();
                self.transport.send(&ser_msg).unwrap();
            }
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
        let data = "Hello".to_string();

        let mut runtime = PeerRuntime::new(
            EmulatedByteChannel::default(),
            512
        );

        let mut channel = runtime.build_channel(10);

        channel.send.send(data);

        runtime.flush();
        runtime.flush();
        runtime.flush();

        let msg = channel.recv.recv().unwrap();

        assert_eq!(msg, "Hello");

    }
}