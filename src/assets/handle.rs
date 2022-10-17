use std::any::TypeId;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;
use crate::asset_server::{Asset, AssetServerGlobal};

pub type HandleId = usize;

pub struct Handle<T : Asset> {
    idx : HandleId,
    marker : PhantomData<T>,
    asset_server : Arc<AssetServerGlobal>,
    strong : bool
}

impl<T : Asset> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self.strong == other.strong
    }
}

impl<T : Asset> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);
    }
}

impl<T> Handle<T> where T : Asset {

    pub fn new(idx : HandleId, asset_server : Arc<AssetServerGlobal>, strong : bool) -> Self {
        let res = Self {
            idx,
            marker : PhantomData::default(),
            asset_server,
            strong
        };
        if strong {
            res.incerase_counter();
        }
        res
    }

    fn incerase_counter(&self) {
        self.asset_server.create_queue.lock()
            .unwrap().push(self.idx);
    }

    pub fn get_untyped(&self) -> HandleUntyped {
        if self.strong {
            self.incerase_counter();
        }

        HandleUntyped {
            idx: self.idx,
            tp: TypeId::of::<T>(),
            asset_server: self.asset_server.clone(),
            strong : self.strong
        }
    }

    pub fn get_idx(&self) -> HandleId {
        self.idx
    }

    pub fn get_weak(&self) -> Handle<T> {
        Handle::new( self.idx, self.asset_server.clone(), false)
    }
}

impl<T> Drop for Handle<T> where T : Asset {
    fn drop(&mut self) {
        if self.strong {
            self.asset_server.destroy_queue.lock().unwrap().push(self.idx);
        }
    }
}

impl<T> Clone for Handle<T> where T : Asset {
    fn clone(&self) -> Self {
        Handle::new(self.idx, self.asset_server.clone(), self.strong)
    }
}

pub struct HandleUntyped {
    idx : HandleId,
    tp : TypeId,
    asset_server : Arc<AssetServerGlobal>,
    strong : bool
}

impl HandleUntyped {

    pub fn new(idx : HandleId, tp : TypeId, asset_server : Arc<AssetServerGlobal>, strong : bool) -> Self {
        let res = Self {
            idx,
            tp,
            asset_server,
            strong
        };
        if res.strong {
            res.incerase_counter();
        }
        res
    }

    pub fn get_strong(&self) -> Self {
        HandleUntyped::new(self.idx, self.tp, self.asset_server.clone(), true)
    }

    pub fn get_typed<T : Asset>(&self) -> Handle<T> {
        Handle::<T>::new(self.idx, self.asset_server.clone(), self.strong)
    }

    fn incerase_counter(&self) {
        self.asset_server.create_queue.lock()
            .unwrap().push(self.idx);
    }

    pub fn get_idx(&self) -> HandleId {
        self.idx
    }
}

impl Drop for HandleUntyped {
    fn drop(&mut self) {
        if self.strong {
            self.asset_server.destroy_queue.lock().unwrap().push(self.idx);
        }
    }
}

impl Clone for HandleUntyped {
    fn clone(&self) -> Self {
        HandleUntyped::new(self.idx, self.tp, self.asset_server.clone(), self.strong)
    }
}
