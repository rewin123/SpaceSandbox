use bevy::utils::{HashMap, Instant, HashSet};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use bincode::*;
use std::{sync::mpsc::*, time::Duration};
use serde::*;

use super::channel::{ByteChannel, EmulatedByteChannel};

pub type ChannelID = u16;

pub trait ByteMsgs {
    fn msg_to_send(&mut self) -> Option<Vec<u8>>;
    fn recv_msg(&mut self, msg : Vec<u8>);
    fn update(&mut self);
}

#[derive(Serialize, Deserialize)]
pub struct ChannelMsg {
    pub channel : ChannelID,
    pub data : Vec<u8>
}

pub struct MessageChannel<T> {
    pub recv : Receiver<T>,
    pub send : Sender<T>,
}

pub type MsgID = u32;

#[derive(Serialize, Deserialize)]
pub enum ReliableMsg {
    GotMsg(MsgID),
    Msg(MsgID, Vec<u8>)
}

#[derive(Clone)]
pub struct ReliableCache {
    pub time : Instant,
    pub msg : Vec<u8>
}

pub struct ReliableTransform<T> {
    pub to_bytes : Receiver<T>,
    pub from_bytes : Sender<T>,
    pub timeout : Duration,
    pub got_timeout : Duration,
    pub buffer : HashMap<MsgID, ReliableCache>,
    pub msg_indexer : MsgID,
    pub rollback : Vec<ReliableMsg>,
    pub gotted : HashMap<MsgID, Instant>
}

impl<T : Serialize + DeserializeOwned + 'static> ReliableTransform<T> {
    fn get_msg_id(&mut self) -> MsgID {
        let id = self.msg_indexer;
        self.msg_indexer = ((self.msg_indexer as u64 + 1) % u32::MAX as u64) as u32;
        id
    }
}

impl<T : Serialize + DeserializeOwned + 'static> ByteMsgs for ReliableTransform<T> {

    fn update(&mut self) {
        let now = Instant::now();
        
        self.gotted = self.gotted.iter().filter(|(_, time)| {
            (now - **time) < self.got_timeout 
        }).map(|(idx, time)| (*idx, *time))
        .collect::<HashMap<_,_>>();
    }

    fn msg_to_send(&mut self) -> Option<Vec<u8>> {
        if let Ok(data) = self.to_bytes.try_recv() {
            let cache = ReliableCache {
                time : Instant::now(),
                msg : bincode::serialize(&data).unwrap()
            };

            let idx = self.get_msg_id();

            self.buffer.insert(idx, cache.clone());

            Some(bincode::serialize(&ReliableMsg::Msg(idx, cache.msg)).unwrap())
        } else {
            let now = Instant::now();
            for (idx, cache) in &mut self.buffer {
                if (now - cache.time) > self.timeout {
                    //resend
                    cache.time = now.clone();
                    return Some(bincode::serialize(&ReliableMsg::Msg(*idx, cache.msg.clone())).unwrap());
                }
            }

            if self.rollback.len() > 0 {
                let msg = self.rollback.remove(0);
                return Some(bincode::serialize(&msg).unwrap());
            }

            None
        }
    }

    fn recv_msg(&mut self, msg : Vec<u8>) {
        let rel_msg : ReliableMsg = bincode::deserialize(&msg).unwrap();
        match rel_msg {
            ReliableMsg::GotMsg(idx) => {
                self.buffer.remove(&idx);
            },
            ReliableMsg::Msg(idx, data) => {
                self.rollback.push(ReliableMsg::GotMsg(idx));
                self.rollback.push(ReliableMsg::GotMsg(idx));
                if !self.gotted.contains_key(&idx) {
                    self.gotted.insert(idx, Instant::now());
                    let data_t : T = bincode::deserialize(&data).unwrap();
                    self.from_bytes.send(data_t);
                } 
            },
        }
    }
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

    fn update(&mut self) {

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

    pub fn build_reliable_channel<D : Serialize + DeserializeOwned + 'static>(&mut self, ch : ChannelID) -> MessageChannel<D> {
        let (to_net_send, to_net_recv) = channel();
        let (from_net_send, from_net_recv) = channel();
        
        let msg_ch = MessageChannel::<D> {
            recv : from_net_recv,
            send : to_net_send,
        };

        let transfer = ReliableTransform::<D> {
            to_bytes : to_net_recv,
            from_bytes : from_net_send,
            timeout: Duration::from_millis(10),
            got_timeout: Duration::from_secs(1),
            buffer: HashMap::new(),
            msg_indexer: 0,
            rollback: vec![],
            gotted: HashMap::new(),
        };

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

    pub fn flush(&mut self) {

        //update channels
        for (_, ch) in &mut self.channels {
            ch.update();
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

        runtime.flush();
        runtime.flush();
        runtime.flush();

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
            runtime.flush();
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
            runtime.flush();
        }

        let msg = channel.recv.recv().unwrap();

        assert_eq!(msg, "Hello");
    }


}