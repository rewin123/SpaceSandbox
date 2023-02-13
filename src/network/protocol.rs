use bevy::utils::{HashMap, Instant, HashSet};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use bincode::*;
use std::{sync::mpsc::*, time::Duration, hash::Hash};
use serde::*;

use super::channel::{ByteChannel, EmulatedByteChannel};
use super::channel_protocol::*;

pub type ChannelID = u16;

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

    pub fn build_large_channel<D : Serialize + DeserializeOwned + 'static>(&mut self, ch : ChannelID) -> MessageChannel<D> {
        let (msg_ch, transfer) = LargeMessageTransform::<D>::build_channel();

        if self.channels.contains_key(&ch) {
            panic!("Channel already exist!");
        }

        self.channels.insert(ch, Box::new(transfer));

        msg_ch
    }

    pub fn build_reliable_channel<D : Serialize + DeserializeOwned + 'static>(&mut self, ch : ChannelID) -> MessageChannel<D> {
        let (msg_ch, transfer) = ReliableTransform::<D>::build_channel();

        if self.channels.contains_key(&ch) {
            panic!("Channel already exist!");
        }

        self.channels.insert(ch, Box::new(transfer));

        msg_ch
    }

    pub fn build_channel<D : Serialize + DeserializeOwned + 'static>(&mut self, ch : ChannelID) -> MessageChannel<D> {
        let (to_net_send, to_net_recv) = channel();
        let (from_net_send, from_net_recv) = channel();
        
        let msg_ch = MessageChannel::<D> {
            recv : from_net_recv,
            send : to_net_send,
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

    pub fn flush(&mut self, dt : Duration) {

        //update channels
        for (_, ch) in &mut self.channels {
            ch.update(dt.clone());
        }

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
                if ser_msg.len() > self.buffer_size {
                    panic!("Message with size {} longer buffer size limit {}", ser_msg.len(), self.buffer_size);
                }
                self.transport.send(&ser_msg).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::channel::*;

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

        runtime.flush(Duration::from_secs(1));
        runtime.flush(Duration::from_secs(1));
        runtime.flush(Duration::from_secs(1));

        let msg = channel.recv.recv().unwrap();

        assert_eq!(msg, "Hello");
    }

    #[test]
    fn reliable_channel_test() {
        let data = "Hello".to_string();

        let mut runtime = PeerRuntime::new(
            EmulatedByteChannel::default(),
            512
        );

        let mut channel = runtime.build_reliable_channel(10);

        channel.send.send(data);

        for idx in 0..3 {
            runtime.flush(Duration::from_secs(1));
        }

        let msg = channel.recv.recv().unwrap();

        assert_eq!(msg, "Hello");
    }

    #[test]
    fn netowrk_loss_test() {
        let data = "Hello".to_string();

        let mut newtork = EmulatedNetworkChannel::default();

        let mut runtime = PeerRuntime::new(
            newtork,
            512
        );

        let mut channel = runtime.build_reliable_channel(10);

        channel.send.send(data);

        for _ in 0..100 {
            runtime.flush(Duration::from_secs(1));
        }

        let msg = channel.recv.try_recv().unwrap();

        assert_eq!(msg, "Hello");
    }

    #[test]
    fn netowrk_loss_large_test() {
        let data = "Hello".to_string().repeat(100);

        let mut newtork = EmulatedNetworkChannel::default();

        let mut runtime = PeerRuntime::new(
            newtork,
            512
        );

        let mut channel = runtime.build_large_channel(10);

        channel.send.send(data.clone());

        for _ in 0..1000 {
            runtime.flush(Duration::from_secs(1));
        }

        let msg = channel.recv.try_recv().unwrap();

        assert_eq!(msg, data);
    }


}