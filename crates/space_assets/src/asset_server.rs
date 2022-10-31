use std::any::Any;
use std::{any::TypeId, iter, marker::PhantomData};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use byteorder::ByteOrder;
use downcast_rs::{DowncastSync, impl_downcast};
use space_core::{RenderBase, SpaceResult};
use space_core::TaskServer;
use std::hash::Hash;
use std::num::NonZeroU32;
use wgpu::util::DeviceExt;
use crate::asset_holder::AssetHolder;
use crate::AssetPath;
use crate::handle::*;
use crate::mesh::TextureBundle;
use crate::mipmap_generator::MipmapGenerator;

pub trait Asset : DowncastSync {

}

impl_downcast!(sync Asset);


pub struct AssetServerDecriptor {
    pub render : Arc<RenderBase>
}


#[derive(Default)]
pub struct AssetServerGlobal {
    pub destroy_queue : Mutex<Vec<HandleId>>,
    pub create_queue : Mutex<Vec<HandleId>>,
    pub background_loading : Mutex<Vec<(HandleUntyped, Arc<dyn Asset>)>>,
    pub mark_to_update : Mutex<Vec<HandleId>>
}

pub struct AssetServer {
    root_path : String,
    assets : HashMap<HandleId, AssetHolder>,
    loaded_assets : HashMap<String, HandleUntyped>,
    render : Arc<RenderBase>,
    counter : HandleId,
    memory_holder : Arc<AssetServerGlobal>,
    pub default_color : Arc<TextureBundle>,
    pub default_normal : Arc<TextureBundle>,
    task_server : Arc<TaskServer>,
}

impl AssetServer {
    pub fn new(
            render : &Arc<RenderBase>,
            task_server : &Arc<TaskServer>) -> AssetServer {

        let def_color = {
            let tex_color = render.device.create_texture_with_data(
                &render.queue, &wgpu::TextureDescriptor {
                    label: Some("default color texture"),
                    size: wgpu::Extent3d {width : 1, height : 1, depth_or_array_layers : 1},
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                }, 
                &[255, 255, 255, 255]);

            let s_color = tex_color.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("default color sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });
            TextureBundle {
                texture: tex_color,
                view: s_color,
                sampler: sampler,
            }
        };

        let def_normal = {
            let tex_color = render.device.create_texture_with_data(
                &render.queue, &wgpu::TextureDescriptor {
                    label: Some("default color texture"),
                    size: wgpu::Extent3d {width : 1, height : 1, depth_or_array_layers : 1},
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                }, 
                &[255 / 2, 255 / 2, 255, 255]);

            let s_color = tex_color.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("default color sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });
            TextureBundle {
                texture: tex_color,
                view: s_color,
                sampler: sampler,
            }
        };

        Self {
            root_path : "res".to_string(),
            render : render.clone(),
            assets : HashMap::new(),
            counter : 0,
            memory_holder : Arc::new(AssetServerGlobal::default()),
            default_color : Arc::new(def_color),
            task_server : task_server.clone(),
            default_normal : Arc::new(def_normal),
            loaded_assets : HashMap::new()
        }
    }

    pub fn get_file_text(&self, path : &AssetPath) -> SpaceResult<String> {
        match path {
            AssetPath::GlobalPath(path) => {
                Ok(std::fs::read_to_string(path)?)
            },
            AssetPath::Binary(bytes) => {
                Ok(String::from_utf8(bytes.clone())?)
            },
            AssetPath::Text(text) => Ok(text.clone()),
        }
    }

    pub fn sync_tick(&mut self) {
        {
            let mut add = self.memory_holder.create_queue.lock().unwrap();
            for a in add.iter() {
                if let Some(h) = self.assets.get_mut(&a) {
                    h.inc_counter();
                }
            }
            add.clear();
        } 
        
        {
            let mut add = self.memory_holder.background_loading.lock().unwrap();
            for idx in 0..add.len() {
                let (handle, data) = &add[idx];
                if let Some(h) = self.assets.get_mut(&handle.get_idx()) {
                    h.update_data(data.clone(), &self.memory_holder);
                }
            }
            add.clear();
        }

        //rebuild part
        {
            let mut add = self.memory_holder.mark_to_update.lock().unwrap();
            for handle in add.iter() {
                if let Some(val) = self.assets.get_mut(&handle) {
                    val.set_rebuild(true);
                }
            }
            add.clear();
        }

        {
            let mut add = self.memory_holder.destroy_queue.lock().unwrap();
            for a in add.iter() {
                if let Some(h) = self.assets.get_mut(&a) {
                    h.dec_counter();
                }
            }
            add.clear();
        }
    }

    pub fn get_files_by_ext(&self, ext : String) -> Vec<String> {
        let path = PathBuf::from(self.root_path.clone());
        self.get_files_by_ext_from_folder(path, ext)
    }

    pub fn get_files_by_ext_from_folder(&self, path : PathBuf, ext : String) -> Vec<String> {
        if path.is_dir() {
            let mut res = vec![];
            for file in path.read_dir().unwrap() {
                if let Ok(entry) = file {
                    if entry.path().is_file() {
                        if let Some(entry_ext) = entry.path().extension() {
                            if entry_ext.to_str().unwrap().to_string() == ext {
                                res.push(entry.path().to_str().unwrap().to_string());
                            }
                        }
                    } else if entry.path().is_dir() {
                        res.extend(self.get_files_by_ext_from_folder(entry.path(), ext.clone()));
                    }
                }
            }
            res
        } else {
            if path.is_file() {
                if let Some(entry_ext) =path.extension() {
                    if entry_ext.to_str().unwrap().to_string() == ext {
                        return vec![path.to_str().unwrap().to_string()];
                    }
                }
            }
            vec![]
        }
    }

    pub fn get<T : Asset>(&self, handle : &Handle<T>) -> Option<Arc<T>> {
        if let Some(val) = self.assets.get(&handle.get_idx()) {
           if let Ok(res) = val.get().clone().downcast_arc::<T>() {
               Some(res)
           } else {
               None
           }
        } else {
            None
        }
    }

    pub fn get_version<T : Asset>(&self, handle : &Handle<T>) -> Option<u32> {
        if let Some(val) = self.assets.get(&handle.get_idx()) {
            Some(val.get_version())
        } else {
            None
        }
    }

    pub fn get_untyped<T : Asset>(&self, handle : &HandleUntyped) -> Option<&T> {
        if let Some(val) = self.assets.get(&handle.get_idx()) {
           val.get().downcast_ref::<T>()
        } else {
            None
        }
    }

    pub fn new_asset<T : Asset>(&mut self, val : Arc<T>) -> Handle<T> {
        let holder = AssetHolder::new(val);
        self.counter += 1;
        self.assets.insert(self.counter, holder);
        Handle::new(self.counter, self.memory_holder.clone(), true)
    }

    pub fn load_color_texture(&mut self, path : String, gamma : bool) -> Handle<TextureBundle> {

        if let Some(handle) = self.loaded_assets.get(&path) {
            if let Some(val) = self.get_untyped::<TextureBundle>(&handle) {
                return handle.get_strong().get_typed();
            }
        }

        self.counter += 1;

        let copy_index = self.counter;
        let handler = self.new_asset(self.default_color.clone());

        let untyped = handler.get_untyped();
        let render = self.render.clone();
        let back = self.memory_holder.clone();

        self.loaded_assets.insert(path.clone(), handler.get_weak().get_untyped());

        let format = {
            if gamma {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            }
        };
        
        self.task_server.spawn(&format!("Loading {}", path).to_string(),move || {

            let image = image::open(path)
                .map(|img| img.to_rgba())
                .expect("unable to open image");
            let (width, height) = image.dimensions();

            let mipcount = (height.max(width) as f32).log2() as u32 + 1;

            let padded_width = ((width * 4) / 256 + 1) * 256;
            let mut padded_buffer = vec![0u8; (padded_width * height * 4) as usize];

            for y in 0..height {
                for x in 0..width {
                    let pix = image.get_pixel(x, y);
                    let shift = ((y * padded_width + x) * 4) as usize;
                    padded_buffer[shift] = pix.0[0];
                    padded_buffer[shift + 1] = pix.0[1];
                    padded_buffer[shift + 2] = pix.0[2];
                    padded_buffer[shift + 3] = pix.0[3];
                }
            }


            let tex_color = render.device.create_texture(
                &wgpu::TextureDescriptor {
                    label: Some("default color texture"),
                    size: wgpu::Extent3d {width, height, depth_or_array_layers : 1},
                    mip_level_count: mipcount,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::COPY_SRC,
                });

            let src_buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &padded_buffer,
                usage: wgpu::BufferUsages::COPY_SRC
            });
    
            let s_color = tex_color.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("default color sampler"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                lod_min_clamp: 0.0,
                lod_max_clamp: mipcount as f32,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });

            let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            });

            let mut layout = wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(padded_width * 4),
                rows_per_image: None
            };

            encoder.copy_buffer_to_texture(wgpu::ImageCopyBufferBase {
                buffer: &src_buffer,
                layout: layout
            }, wgpu::ImageCopyTextureBase {
                texture: &tex_color,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers : 1
            });

            render.queue.submit(iter::once(encoder.finish()));

            MipmapGenerator::generate(
                &render,
                &tex_color,
                width,
                height,
                mipcount,
                format);

            let bundle = TextureBundle {
                texture : tex_color,
                view : s_color,
                sampler
            };

            back.background_loading.lock().unwrap()
                .push((untyped, Arc::new(bundle)));
        });

        handler
    }

}