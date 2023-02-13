use bevy::utils::{HashMap, Instant, HashSet};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use bincode::*;
use std::{sync::mpsc::*, time::Duration, hash::Hash};
use serde::*;

use super::channel::{ByteChannel, EmulatedByteChannel};

pub type ChannelID = u16;

pub trait ByteMsgs {
    fn msg_to_send(&mut self) -> Option<Vec<u8>>;
    fn recv_msg(&mut self, msg : Vec<u8>);
    fn update(&mut self, dt : Duration);
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
    pub gotted : HashMap<MsgID, Instant>,
    pub time : Instant
}

impl<T : Serialize + DeserializeOwned + 'static> ReliableTransform<T> {
    fn get_msg_id(&mut self) -> MsgID {
        let id = self.msg_indexer;
        self.msg_indexer = ((self.msg_indexer as u64 + 1) % u32::MAX as u64) as u32;
        id
    }

    fn build_channel() -> (MessageChannel<T>, ReliableTransform<T>) {
        let (to_net_send, to_net_recv) = channel();
        let (from_net_send, from_net_recv) = channel();
        
        let msg_ch = MessageChannel::<T> {
            recv : from_net_recv,
            send : to_net_send,
        };

        let transfer = ReliableTransform::<T> {
            to_bytes : to_net_recv,
            from_bytes : from_net_send,
            timeout: Duration::from_millis(10),
            got_timeout: Duration::from_secs(1),
            buffer: HashMap::new(),
            msg_indexer: 0,
            rollback: vec![],
            gotted: HashMap::new(),
            time : Instant::now()
        };

        (msg_ch, transfer)
    }
}

impl<T : Serialize + DeserializeOwned + 'static> ByteMsgs for ReliableTransform<T> {

    fn update(&mut self, dt : Duration) {
        self.time += dt;
        
        self.gotted = self.gotted.iter().filter(|(_, time)| {
            (self.time - **time) < self.got_timeout 
        }).map(|(idx, time)| (*idx, *time))
        .collect::<HashMap<_,_>>();
    }

    fn msg_to_send(&mut self) -> Option<Vec<u8>> {
        if let Ok(data) = self.to_bytes.try_recv() {
            let cache = ReliableCache {
                time : self.time.clone(),
                msg : bincode::serialize(&data).unwrap()
            };

            let idx = self.get_msg_id();

            self.buffer.insert(idx, cache.clone());

            Some(bincode::serialize(&ReliableMsg::Msg(idx, cache.msg)).unwrap())
        } else {
            let now = self.time.clone();
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
                    self.gotted.insert(idx, self.time.clone());
                    let data_t : T = bincode::deserialize(&data).unwrap();
                    self.from_bytes.send(data_t);
                } 
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LargeMessage {
    SetObjectDesc{obj_id : MsgID, part_count : usize, part_size : usize},
    ObjectPart{obj_id : MsgID, part_id : usize, data : Vec<u8>},
    GotObjectDesc(MsgID)
}

pub enum LargeSendState {
    SentDesc,
    Loading,
    Finish
}

pub struct LargeSend {
    pub desc : LargeMessage,
    pub parts : Vec<LargeMessage>,
    pub state : LargeSendState
}

pub enum LargeRecvState {
    Loading,
    Finished
}

pub struct LargeRecv {
    pub state : LargeRecvState,
    pub parts : HashMap<usize, Vec<u8>>,
    pub part_count : usize
}

impl LargeRecv {
    fn collect_obj<T>(&self) -> T where T : DeserializeOwned {
        let mut data = vec![];
        for idx in 0..self.part_count {
            data.extend(self.parts[&idx].iter());
        }
        let obj = bincode::deserialize(&data).unwrap();
        obj
    }
}

pub struct LargeMessageTransform<T> {
    pub channel : MessageChannel<LargeMessage>,
    pub reliable_channel : ReliableTransform<LargeMessage>,
    pub to_bytes : Receiver<T>,
    pub from_bytes : Sender<T>,
    pub split_size : usize,
    pub sends : HashMap<MsgID, LargeSend>,
    pub recvs : HashMap<MsgID, LargeRecv>,
    pub obj_indexer : MsgID
}

impl<T : Serialize + DeserializeOwned + 'static> LargeMessageTransform<T> {
    pub fn get_obj_id(&mut self) -> MsgID {
        let res = self.obj_indexer;
        self.obj_indexer.wrapping_add(1);
        res
    }

    fn build_channel() -> (MessageChannel<T>, LargeMessageTransform<T>) {
        let (reliable_msg, reliable_transfer) = ReliableTransform::<LargeMessage>::build_channel();

        let (to_net_send, to_net_recv) = std::sync::mpsc::channel();
        let (from_net_send, from_net_recv) = std::sync::mpsc::channel();

        let msg_ch = MessageChannel::<T> {
            recv: from_net_recv,
            send: to_net_send,
        };

        let transfer = LargeMessageTransform::<T> {
            channel: reliable_msg,
            reliable_channel: reliable_transfer,
            to_bytes: to_net_recv,
            from_bytes: from_net_send,
            split_size: 256,
            sends: HashMap::new(),
            recvs: HashMap::new(),
            obj_indexer: 0,
        };

        (msg_ch, transfer)
    }

    fn recv_msgs(&mut self) {
        while let Ok(msg) = self.channel.recv.try_recv() {
            match msg {
                LargeMessage::SetObjectDesc { obj_id, part_count, part_size } => {
                    let large = LargeRecv {
                        state: LargeRecvState::Loading,
                        parts: HashMap::new(),
                        part_count,
                    };
                    self.recvs.insert(obj_id, large);
                    self.channel.send.send(LargeMessage::GotObjectDesc(obj_id));
                },
                LargeMessage::ObjectPart { obj_id, part_id, data } => {
                    let mut need_delete = false;
                    if let Some(large) = self.recvs.get_mut(&obj_id) {
                        large.parts.insert(part_id, data);

                        if large.parts.len() == large.part_count {
                            let obj : T = large.collect_obj();
                            large.state = LargeRecvState::Finished;
                            self.from_bytes.send(obj);
                            need_delete = true;
                        }
                    }

                    if need_delete {
                        self.recvs.remove(&obj_id);
                    }


                },
                LargeMessage::GotObjectDesc(id) => {
                    if let Some(large) = self.sends.get_mut(&id) {
                        for idx in 0..large.parts.len() {
                            self.channel.send.send(large.parts[idx].clone());
                        }
                    }
                    self.sends.remove(&id);
                },
            }
        }
    }
}

impl<T : Serialize + DeserializeOwned + 'static> ByteMsgs for LargeMessageTransform<T> {
    fn update(&mut self, dt : Duration) {
        self.reliable_channel.update(dt.clone());

        self.recv_msgs();

        while let Ok(obj) = self.to_bytes.try_recv() {
            let obj_id = self.get_obj_id();
            let data = bincode::serialize(&obj).unwrap();
            let mut splits = vec![];
            let mut split_idx = 0;
            while split_idx < data.len() {
                let end_split = (split_idx + self.split_size).min(data.len());
                splits.push(data[split_idx..end_split].to_vec());
                split_idx = end_split;
            }

            let mut parts = vec![];
            for (idx, split) in splits.iter().enumerate() {
                let part = LargeMessage::ObjectPart { 
                    obj_id, 
                    part_id: idx, 
                    data: split.clone() 
                };
                parts.push(part);
            }

            let desc = LargeMessage::SetObjectDesc { 
                obj_id,
                part_count: parts.len(), 
                part_size: self.split_size 
            };

            self.channel.send.send(desc.clone());

            let large = LargeSend {
                desc,
                parts,
                state: LargeSendState::SentDesc,
            };

            self.sends.insert(obj_id, large);
        }
    }

    fn msg_to_send(&mut self) -> Option<Vec<u8>> {
        self.reliable_channel.msg_to_send()
    }

    fn recv_msg(&mut self, msg : Vec<u8>) {
        self.reliable_channel.recv_msg(msg);
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

    fn update(&mut self, dt : Duration) {

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