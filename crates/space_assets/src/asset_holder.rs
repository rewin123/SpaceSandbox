use std::sync::Arc;
use crate::asset_server::{Asset, AssetServerGlobal};
use crate::handle::HandleUntyped;

pub struct AssetHolder {
    ptr : Box<dyn Asset>,
    count : i32,
    version : u32,
    watchers : Vec<HandleUntyped>,
    need_to_rebuild : bool
}

impl AssetHolder {
    pub fn new(ptr : Box<dyn Asset>) -> Self {
        AssetHolder {
            ptr,
            count : 0,
            version : 0,
            watchers : vec![],
            need_to_rebuild : false
        }
    }

    pub fn get(&self) -> &Box<dyn Asset> {
        &self.ptr
    }

    pub fn get_mut(&mut self) -> &mut Box<dyn Asset> {
        &mut self.ptr
    }

    pub fn inc_counter(&mut self) {
        self.count += 1;
    }

    pub fn dec_counter(&mut self) -> bool {
        self.count -= 1;
        self.count <= 0
    }

    pub fn update_data(&mut self, ptr : Box<dyn Asset>, core : &Arc<AssetServerGlobal>) {
        self.ptr = ptr;
        self.version += 1;

        let mut lock = core.mark_to_update.lock().unwrap();
        for w in &self.watchers {
            lock.push(w.get_idx());
        }
    }

    pub fn set_rebuild(&mut self, rebuild : bool) {
        self.need_to_rebuild = rebuild;
    }

    pub fn get_rebuild(&mut self) -> bool {
        self.need_to_rebuild
    }

    pub fn get_version(&self) -> u32 {
        self.version
    }
}