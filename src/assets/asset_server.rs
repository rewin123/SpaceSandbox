use std::any::TypeId;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use ash::vk::BufferUsageFlags;
use byteorder::ByteOrder;
use gltf::buffer::Source;
use gltf::json::accessor::ComponentType;
use gltf::Semantic;
use crate::{BufferSafe, Game, GPUMesh, Material, RenderModel, TextureServer, TextureType};
use log::*;
use crate::wavefront::load_gray_obj_now;

pub struct BaseModels {
    pub sphere : Arc<GPUMesh>
}

pub struct AssetServer {
    root_path : String,
    pub texture_server : TextureServer,
    pub base_models : BaseModels
}

impl AssetServer {
    pub fn new(game : &Game) -> AssetServer {
        let mut texture_server = TextureServer::new(
            &game.gb, &game.pools, game.task_server.clone());

        let base_models = BaseModels {
            sphere : load_gray_obj_now(
                &game.gb,
                "res/base_models/sphere.obj".to_string())
                .unwrap()[0].clone()
        };

        Self {
            root_path : "res".to_string(),
            texture_server,
            base_models
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

    pub fn load_static_gltf(&mut self, game : &mut Game, path : String) {

        let mut scene = vec![];

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

        let mut images = vec![];

        for img_meta in sponza.images() {
            match img_meta.source() {
                gltf::image::Source::Uri {uri, mime_type} => {
                    let path = format!("{}/{}", base, uri);
                    info!("Loading texture {} ...", path);

                    images.push(self.texture_server.load_new_texture(path, TextureType::Color));
                }
                _ => {
                    panic!("Not supported source for texture");
                }
            }
        }

        let mut meshes = vec![];

        for m in sponza.meshes() {
            let mut sub_models = vec![];
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


                let mut pos_buffer = BufferSafe::new(
                    &game.gb.allocator,
                    pos.len() as u64 * 4,
                    BufferUsageFlags::VERTEX_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu).unwrap();
                let mut normal_buffer = BufferSafe::new(
                    &game.gb.allocator,
                    pos.len() as u64 * 4,
                    BufferUsageFlags::VERTEX_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu).unwrap();

                if tangent.len() == 0 {
                    tangent = vec![0.0f32; pos.len()];
                    info!("No tangent!");
                }

                let mut tangent_buffer = BufferSafe::new(
                    &game.gb.allocator,
                    tangent.len() as u64 * 4,
                    BufferUsageFlags::VERTEX_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu
                ).unwrap();

                let mut index_buffer = BufferSafe::new(
                    &game.gb.allocator,
                    indices.len() as u64 * 4,
                    BufferUsageFlags::INDEX_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu
                ).unwrap();

                if uv.len() == 0 {
                    uv = vec![0.0f32; pos.len() / 3 * 2];
                }

                let mut uv_buffer = BufferSafe::new(
                    &game.gb.allocator,
                    uv.len() as u64 * 4,
                    BufferUsageFlags::VERTEX_BUFFER,
                    gpu_allocator::MemoryLocation::CpuToGpu
                ).unwrap();

                pos_buffer.fill(&pos).unwrap();
                normal_buffer.fill(&normals).unwrap();
                index_buffer.fill(&indices).unwrap();
                uv_buffer.fill(&uv).unwrap();
                tangent_buffer.fill(&tangent).unwrap();

                let mesh = GPUMesh {
                    pos_data: pos_buffer,
                    normal_data: normal_buffer,
                    index_data: index_buffer,
                    tangent_data: tangent_buffer,
                    uv_data : uv_buffer,
                    vertex_count: indices.len() as u32,
                    name: m.name().unwrap_or("").to_string()
                };

                let normal_tex;
                if let Some(tex) = p.material().normal_texture() {
                    normal_tex = images[tex.texture().index()].clone();
                    self.texture_server.textures.insert(normal_tex.server_index, self.texture_server.default_normal_texture.clone());
                } else {
                    normal_tex = self.texture_server.get_default_normal_texture();
                }

                let metallic_roughness;
                if let Some(tex) = p.material().pbr_metallic_roughness().metallic_roughness_texture() {
                    metallic_roughness = images[tex.texture().index()].clone();
                } else {
                    metallic_roughness = self.texture_server.get_default_color_texture();
                }

                let material = {
                    match p.material().pbr_specular_glossiness() {
                        Some(v) => {

                            let color;
                            if let Some(tex) = v.diffuse_texture() {
                                color = images[tex.texture().index()].clone()
                            } else {
                                color = self.texture_server.get_default_color_texture();
                            }

                            Material {
                                color,
                                normal : normal_tex,
                                metallic_roughness: metallic_roughness
                            }
                        }
                        None => {
                            Material {
                                color : images[p.material().pbr_metallic_roughness().base_color_texture().unwrap().texture().index()].clone(),
                                normal : normal_tex,
                                metallic_roughness: metallic_roughness
                            }
                        }
                    }
                };

                let model = RenderModel::new(&game.gb.allocator,
                                             Arc::new(mesh),
                                             material);
                sub_models.push(model);
            }
            meshes.push(sub_models);
        }

        for n in sponza.nodes() {
            let matrix = n.transform().matrix();
            if let Some(mesh) = n.mesh() {
                for rm in &mut meshes[mesh.index()] {
                    rm.add_matrix(&matrix);
                }
            } else {
                for child in n.children() {
                    if let Some(mesh) = child.mesh() {
                        for rm in &mut meshes[mesh.index()] {
                            rm.add_matrix(&matrix);
                        }
                    }
                }
            }
        }

        scene = meshes.into_iter().flatten().collect();

        for rm in &mut scene {
            rm.update_instance_buffer().unwrap();
        }

        game.render_server.render_models.extend(scene);
    }
}