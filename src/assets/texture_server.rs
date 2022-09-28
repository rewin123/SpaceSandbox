use std::{collections::HashMap, sync::Weak, sync::{Arc, Mutex}};

use crate::{TextureSafe, GraphicBase, Pools, ApiBase, Pool};
use egui::mutex::RwLock;
use rayon::prelude::*;
use ash::vk;
use crate::task_server::TaskServer;

#[derive(Clone)]
pub struct ServerTexture {
    pub server_index : usize,
    pub texture : Arc<TextureSafe>
}

#[derive(Clone)]
struct TexFormTask {
    data : Vec<u8>,
    index : usize,
    width : u32,
    height : u32
}

pub struct TextureServer {
    pub textures : HashMap<usize, Arc<TextureSafe>>,
    default_texture : Arc<TextureSafe>,
    index : usize,
    update_count : usize,
    waiting_list : Arc<Mutex<Vec<TexFormTask>>>,
    api_base : ApiBase,
    task_server : Arc<TaskServer>
}

impl TextureServer {
    pub fn new(gb : &GraphicBase, pools : &Pools, task_server : Arc<TaskServer>) -> Self {
        Self {
            textures : HashMap::new(),
            index : 0,
            default_texture : Arc::new(TextureSafe::from_raw_data(
                &[150, 150, 150, 255], 
                1, 
                1, 
                &gb.get_api_base(pools)).unwrap()),
            waiting_list : Arc::new(Mutex::new(vec![])),
            api_base : gb.get_api_base(pools),
            update_count : 0,
            task_server
        }
    }

    pub fn load_new_texture(
        &mut self,
        path : String
    ) -> ServerTexture {
        self.index += 1;

        let copy_index = self.index;
        let wait_list = self.waiting_list.clone();
        self.task_server.spawn(&format!("Loading {}", path).to_string(),move || {

            let image = image::open(path)
            .map(|img| img.to_rgba())
            .expect("unable to open image");
            let (width, height) = image.dimensions();

            wait_list.lock().as_mut().unwrap().push(TexFormTask {
                data: image.to_vec(),
                index: copy_index,
                width,
                height
            } );
        });

        self.textures.insert(self.index, self.default_texture.clone());

        ServerTexture { 
            server_index: self.index, 
            texture: self.default_texture.clone()
        }
    }

    pub fn sync_tick(&mut self) {
        let mut waiting_lock = self.waiting_list.lock().unwrap();

        for idx in 0..waiting_lock.len() {
            let s = waiting_lock.get(idx).unwrap().clone();
            let tex = TextureSafe::from_raw_data(
                &s.data, 
                s.width, 
                s.height, 
                &self.api_base).unwrap();

            self.textures.insert(s.index, Arc::new(tex));
            break;
        }
        if waiting_lock.len() > 0 {
        waiting_lock.remove(0);
        }

        // waiting_lock.clear();
    }

    pub fn get_default_color_texture(&mut self) -> ServerTexture {
        let copy_index = self.index;
        self.index += 1;

        self.textures.insert(self.index, self.default_texture.clone());

        ServerTexture {
            server_index: self.index,
            texture: self.default_texture.clone()
        }
    }
}