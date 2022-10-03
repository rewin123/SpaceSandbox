use std::{collections::HashMap, sync::{Arc, Mutex}};
use log::*;

use crate::{TextureSafe, GraphicBase, Pools, ApiBase};
use crate::task_server::TaskServer;

pub struct ServerTexture {
    pub server_index : usize,
    counter : Arc<Mutex<TextureCounter>>
}

impl ServerTexture {
    fn new(idx : usize, counter : &Arc<Mutex<TextureCounter>>) -> Self {
        counter.lock().unwrap().add_item(idx);

        Self {
            server_index : idx,
            counter : counter.clone()
        }
    }
}

impl Clone for ServerTexture {
    fn clone(&self) -> Self {
        ServerTexture::new(self.server_index, &self.counter)
    }
}

impl Drop for ServerTexture {
    fn drop(&mut self) {
        self.counter.lock().unwrap().remove_item(self.server_index);
    }
}

impl ServerTexture {
    pub fn get_texture(&self, server : &TextureServer) -> Arc<TextureSafe> {
        server.textures[&self.server_index].clone()
    }
}

#[derive(Clone)]
struct TexFormTask {
    data : Vec<u8>,
    index : usize,
    width : u32,
    height : u32
}

struct TextureCounter {
    counter : HashMap<usize, i32>,
    destroy_list : Vec<usize>
}

impl TextureCounter {
    fn new_async() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(
            Self {
                counter : HashMap::new(),
                destroy_list : Vec::new()
            }
        ))
    }

    fn remove_item(&mut self, idx : usize) {
        *self.counter.get_mut(&idx).unwrap() -= 1;
        if self.counter[&idx] <= 0 {
            self.counter.remove(&idx);
            self.destroy_list.push(idx);
            info!("Destroy server texture: {}", idx);
        }
    }

    fn add_item(&mut self, idx : usize) {
        if self.counter.contains_key(&idx) {
            *self.counter.get_mut(&idx).unwrap() += 1;
        } else {
            self.counter.insert(idx, 1);
        }
    }

}

pub struct TextureServer {
    pub textures : HashMap<usize, Arc<TextureSafe>>,
    default_texture : Arc<TextureSafe>,
    index : usize,
    waiting_list : Arc<Mutex<Vec<TexFormTask>>>,
    api_base : ApiBase,
    task_server : Arc<TaskServer>,
    counter : Arc<Mutex<TextureCounter>>
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
            task_server,
            counter : TextureCounter::new_async()
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

        ServerTexture::new(self.index, &self.counter)
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

        let mut counter = self.counter.lock().unwrap();
        for del_idx in &counter.destroy_list {
            self.textures.remove(del_idx);
        }
        counter.destroy_list.clear();

        // waiting_lock.clear();
    }

    pub fn get_default_color_texture(&mut self) -> ServerTexture {
        self.index += 1;

        self.textures.insert(self.index, self.default_texture.clone());

        ServerTexture::new(self.index, &self.counter)
    }
}