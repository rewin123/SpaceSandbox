use std::any::Any;
use std::{any::TypeId, marker::PhantomData};
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use byteorder::ByteOrder;
use downcast_rs::{DowncastSync, impl_downcast};
use egui::epaint::ahash::{HashMap, HashMapExt};
use gltf::buffer::Source;
use gltf::json::accessor::ComponentType;
use gltf::Semantic;
use specs::WorldExt;
use crate::task_server::TaskServer;
use crate::{GMesh, GVertex, RenderBase, TextureBundle, Material};
use log::*;
use wgpu::util::DeviceExt;
use specs::*;
use std::hash::Hash;
use crate::asset_holder::AssetHolder;
use crate::handle::*;

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
    default_color : Arc<TextureBundle>,
    default_normal : Arc<TextureBundle>,
    task_server : Arc<TaskServer>
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
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                }, 
                &[0, 0, 255, 255]);

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
            for (handle, data) in add.iter() {
                if let Some(h) = self.assets.get_mut(&handle.get_idx()) {
                    h.update_data(data.clone(), &self.memory_holder);
                    log::info!("New data seted");
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
           match val.get().clone().downcast_arc::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
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

    pub fn get_untyped<T : Asset>(&self, handle : &HandleUntyped) -> Option<Arc<T>> {
        if let Some(val) = self.assets.get(&handle.get_idx()) {
           match val.get().clone().downcast_arc::<T>() {
            Ok(val) => Some(val),
            Err(_) => None,
        }
        } else {
            None
        }
    }

    fn new_asset<T : Asset>(&mut self, val : Arc<T>) -> Handle<T> {
        let holder = AssetHolder::new(val);
        self.counter += 1;
        self.assets.insert(self.counter, holder);
        Handle::new(self.counter, self.memory_holder.clone(), true)
    }

    pub fn load_color_texture(&mut self, path : String) -> Handle<crate::TextureBundle> {

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
        
        self.task_server.spawn(&format!("Loading {}", path).to_string(),move || {

            let image = image::open(path)
                .map(|img| img.to_rgba())
                .expect("unable to open image");
            let (width, height) = image.dimensions();


            let tex_color = render.device.create_texture_with_data(
                &render.queue, &wgpu::TextureDescriptor {
                    label: Some("default color texture"),
                    size: wgpu::Extent3d {width, height, depth_or_array_layers : 1},
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                }, 
                &image);
    
            let s_color = tex_color.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("default color sampler"),
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 0.0,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });

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

    pub fn load_gltf_color_texture(&mut self, base : &String, src : Option<gltf::texture::Info>) -> Handle<TextureBundle> {
        if let Some(tex) = src {
            match tex.texture().source().source() {
                gltf::image::Source::View { view, mime_type } => todo!(),
                gltf::image::Source::Uri { uri, mime_type } => {
                    self.load_color_texture(format!("{}/{}",base, uri))
                },
            }
        } else {
            self.new_asset(self.default_color.clone())
        }
    }

    pub fn load_gltf_normal_texture(&mut self, base : &String, src : Option<gltf::material::NormalTexture>) -> Handle<TextureBundle> {
        if let Some(tex) = src {
            match tex.texture().source().source() {
                gltf::image::Source::View { view, mime_type } => todo!(),
                gltf::image::Source::Uri { uri, mime_type } => {
                    self.load_color_texture(format!("{}/{}",base, uri))
                },
            }
        } else {
            self.new_asset(self.default_normal.clone())
        }
    }

    pub fn wgpu_gltf_load(&mut self, device : &wgpu::Device, path : String, world : &mut specs::World) {

        let sponza = gltf::Gltf::open(&path).unwrap();
        let base = PathBuf::from(&path).parent().unwrap().to_str().unwrap().to_string();

        let mut buffers = vec![];
        for buf in sponza.buffers() {
            match buf.source() {
                Source::Bin => {
                    error!("Bin buffer not supported!");
                }
                Source::Uri(uri) => {
                    info!("Loading buffer {} ...", uri);
                    let mut f = std::fs::File::open(format!("{}/{}", &base, uri)).unwrap();
                    let metadata = std::fs::metadata(&format!("{}/{}", &base, uri)).unwrap();
                    let mut byte_buffer = vec![0; metadata.len() as usize];
                    f.read(&mut byte_buffer).unwrap();
                    buffers.push(byte_buffer);
                }
            }
        }

        // let mut images = vec![];

        // for img_meta in sponza.images() {
        //     match img_meta.source() {
        //         gltf::image::Source::Uri {uri, mime_type} => {
        //             let path = format!("{}/{}", base, uri);
        //             info!("Loading texture {} ...", path);
        //             images.push(self.load_color_texture(path));
        //         }
        //         _ => {
        //             panic!("Not supported source for texture");
        //         }
        //     }
        // }

        for m in sponza.meshes() {
            for p in m.primitives() {
                let mut pos : Vec<f32> = vec![];
                let mut normals : Vec<f32> = vec![];
                let mut uv : Vec<f32> = vec![];
                let mut tangent : Vec<f32> = vec![];

                let indices_acc = p.indices().unwrap();
                let indices_view = indices_acc.view().unwrap();
                let mut indices;

                info!("ind: {:?}", indices_acc.data_type());

                match indices_acc.data_type() {
                    ComponentType::U16 => {
                        indices = vec![0; indices_acc.count()];
                        let buf = &buffers[indices_view.buffer().index()];
                        info!("indices stride: {:?}", indices_view.stride());
                        for idx in 0..indices.len() {
                            let global_idx = idx * indices_view.stride().unwrap_or(2) + indices_view.offset() + indices_acc.offset();
                            indices[idx] = byteorder::LittleEndian::read_u16(&buf[global_idx..(global_idx + 2)]) as u32;
                        }
                    }
                    ComponentType::U32 => {
                        indices = vec![0; indices_acc.count()];
                        let buf = &buffers[indices_view.buffer().index()];
                        info!("indices stride: {:?}", indices_view.stride());
                        for idx in 0..indices.len() {
                            let global_idx = idx * indices_view.stride().unwrap_or(4) + indices_view.offset() + indices_acc.offset();
                            indices[idx] = byteorder::LittleEndian::read_u32(&buf[global_idx..(global_idx + 4)]) as u32;
                        }
                    }
                    _ => {panic!("Unsupported index type!");}
                }

                for (sem, acc) in p.attributes() {
                    // match  { }
                    let view = acc.view().unwrap();
                    let mut data = vec![0.0f32; acc.count() * acc.dimensions().multiplicity()];

                    let stride = view.stride().unwrap_or(acc.data_type().size() * acc.dimensions().multiplicity());

                    let buf = &buffers[view.buffer().index()];

                    for c in 0..acc.count() {
                        for d in 0..acc.dimensions().multiplicity() {
                            let idx = c * acc.dimensions().multiplicity() + d;
                            let global_idx = c * stride + acc.offset() + view.offset() + d * acc.data_type().size();
                            data[idx] = byteorder::LittleEndian::read_f32(&buf[global_idx..(global_idx + 4)]);
                        }
                    }

                    match sem {
                        Semantic::Positions => {
                            pos.extend(data.iter());
                            info!("Pos {}", acc.dimensions().multiplicity());
                            info!("Stride: {}", stride);
                        }
                        Semantic::Normals => {
                            normals.extend(data.iter());
                        }
                        Semantic::Tangents => {
                            tangent.extend(data.iter());
                        }
                        Semantic::Colors(_) => {}
                        Semantic::TexCoords(_) => {
                            uv.extend(data.iter());
                        }
                        Semantic::Joints(_) => {}
                        Semantic::Weights(_) => {}
                        _ => {}
                    }
                }
                info!("Loaded mesh with {} positions and {} normals", pos.len(), normals.len());

                //load diffuse texture

                if tangent.len() == 0 {
                    tangent = vec![0.0f32; pos.len()];
                    info!("No tangent!");
                }

                if uv.len() == 0 {
                    uv = vec![0.0f32; pos.len() / 3 * 2];
                }

                let vertex_count = pos.len() / 3;

                let mut verts = vec![];
                for i in 0..vertex_count {
                    let shift = i * 3;
                    let uv_shift = i * 2;
                    verts.push( GVertex {
                        pos: [pos[shift], pos[shift + 1], pos[shift + 2]],
                        normal: [normals[shift], normals[shift + 1], normals[shift + 2]],
                        tangent: [tangent[shift], tangent[shift + 1], tangent[shift + 2]],
                        uv: [uv[uv_shift], uv[uv_shift + 1]],
                    });
                }
                
                let vert_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gltf vertex buffer"),
                    contents: bytemuck::cast_slice(&verts),
                    usage: wgpu::BufferUsages::VERTEX
                });

                let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("gltf index buffer"),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX
                });

                let model = GMesh {
                    vertex : vert_buffer,
                    index : index_buf,
                    index_count : indices.len() as u32
                };

                let normal = self.load_gltf_normal_texture(&base, p.material().normal_texture());
                
                let color = self.load_gltf_color_texture(&base, p.material().pbr_metallic_roughness().base_color_texture());
                
                let mr = self.load_gltf_color_texture(&base, p.material().pbr_metallic_roughness().metallic_roughness_texture());

                let material = Material {
                    color,
                    normal,
                    metallic_roughness: mr,
                    gbuffer_bind : None,
                    version_sum : 0
                };

                world.create_entity().with(model).with(material).build();
            }
        }
    }
}