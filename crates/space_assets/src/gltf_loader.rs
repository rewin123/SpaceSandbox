use std::io::Read;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use bevy::ecs::world::EntityMut;
use bevy::prelude::Entity;
use byteorder::ByteOrder;
use gltf::animation::Property::Rotation;
use gltf::json::accessor::ComponentType;
use gltf::Semantic;
use wgpu::util::DeviceExt;
use crate::asset_server::SpaceAssetServer;
use crate::handle::Handle;
use crate::{GMeshPtr, Location};
use crate::mesh::{GMesh, GVertex, Material, TextureBundle};


pub trait GltfAssetLoader {
    fn load_gltf_color_texture(&mut self, base : &String, src : Option<gltf::texture::Info>, gamma : bool) -> Handle<TextureBundle>;
    fn load_gltf_normal_texture(&mut self, base : &String, src : Option<gltf::material::NormalTexture>) -> Handle<TextureBundle>;
    fn wgpu_gltf_load(&mut self, device : &wgpu::Device, path : String, world : &mut space_core::bevy::prelude::World) -> Vec<Entity>;
    fn wgpu_gltf_load_cmds(&mut self, device : &wgpu::Device, path : String) -> Vec<MeshBundle>;
}

impl GltfAssetLoader for SpaceAssetServer {

    fn load_gltf_color_texture(&mut self, base : &String, src : Option<gltf::texture::Info>, gamma : bool) -> Handle<TextureBundle> {
        if let Some(tex) = src {
            match tex.texture().source().source() {
                gltf::image::Source::View { view, mime_type } => todo!(),
                gltf::image::Source::Uri { uri, mime_type } => {
                    self.load_color_texture(format!("{}/{}",base, uri), gamma)
                },
            }
        } else {
            self.new_asset(self.default_color.clone())
        }
    }

    fn load_gltf_normal_texture(&mut self, base : &String, src : Option<gltf::material::NormalTexture>) -> Handle<TextureBundle> {
        if let Some(tex) = src {
            match tex.texture().source().source() {
                gltf::image::Source::View { view, mime_type } => todo!(),
                gltf::image::Source::Uri { uri, mime_type } => {
                    self.load_color_texture(format!("{}/{}",base, uri), false)
                },
            }
        } else {
            self.new_asset(self.default_normal.clone())
        }
    }


    fn wgpu_gltf_load_cmds(&mut self, device : &wgpu::Device, path : String) -> Vec<MeshBundle> {
        let mut res = vec![];

        let sponza = gltf::Gltf::open(&path).unwrap();
        let base = PathBuf::from(&path).parent().unwrap().to_str().unwrap().to_string();

        let mut buffers = vec![];
        for buf in sponza.buffers() {
            match buf.source() {
                gltf::buffer::Source::Bin => {

                }
                gltf::buffer::Source::Uri(uri) => {

                    let mut f = std::fs::File::open(format!("{}/{}", &base, uri)).unwrap();
                    let metadata = std::fs::metadata(&format!("{}/{}", &base, uri)).unwrap();
                    let mut byte_buffer = vec![0; metadata.len() as usize];
                    f.read(&mut byte_buffer).unwrap();
                    buffers.push(byte_buffer);
                }
            }
        }

        let mut meshes = vec![];

        for m in sponza.meshes() {
            let mut combined = vec![];
            for p in m.primitives() {
                let mut pos : Vec<f32> = vec![];
                let mut normals : Vec<f32> = vec![];
                let mut uv : Vec<f32> = vec![];
                let mut tangent : Vec<f32> = vec![];

                let indices_acc = p.indices().unwrap();
                let indices_view = indices_acc.view().unwrap();
                let mut indices;

                match indices_acc.data_type() {
                    ComponentType::U16 => {
                        indices = vec![0; indices_acc.count()];
                        let buf = &buffers[indices_view.buffer().index()];
                        for idx in 0..indices.len() {
                            let global_idx = idx * indices_view.stride().unwrap_or(2) + indices_view.offset() + indices_acc.offset();
                            indices[idx] = byteorder::LittleEndian::read_u16(&buf[global_idx..(global_idx + 2)]) as u32;
                        }
                    }
                    ComponentType::U32 => {
                        indices = vec![0; indices_acc.count()];
                        let buf = &buffers[indices_view.buffer().index()];
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
                //load diffuse texture
                let vertex_count = pos.len() / 3;
                let triangle_count = indices.len() / 3;

                if uv.len() == 0 {
                    uv = vec![0.0f32; pos.len() / 3 * 2];
                }

                if tangent.len() == 0 {
                    tangent = vec![0.0f32; pos.len()];
                    println!("[ERROR] No tangents for object!");
                }



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

                let color = self.load_gltf_color_texture(&base, p.material().pbr_metallic_roughness().base_color_texture(), true);

                let mr = self.load_gltf_color_texture(&base, p.material().pbr_metallic_roughness().metallic_roughness_texture(), true);

                let material = Material {
                    color,
                    normal,
                    metallic_roughness: mr,
                    gbuffer_bind : None,
                    version_sum : 0
                };

                combined.push((GMeshPtr {mesh : Arc::new(model)}, material));
            }
            meshes.push(combined);
        }

        for n in sponza.nodes() {
            if let Some(mesh_idx) = n.mesh() {
                let mut location = Location::new(&device);

                let (tr, quat, scale) = n.transform().decomposed();
                let q = nalgebra::Quaternion::new(quat[3], quat[0], quat[1], quat[2]);
                let rot = nalgebra::Rotation::from(nalgebra::UnitQuaternion::from_quaternion(q));
                let (e_x, e_y, e_z) = rot.euler_angles();
                location.pos = tr.into();
                location.rotation = [e_x, e_y, e_z].into();
                location.scale = scale.into();

                for (p, m) in &meshes[mesh_idx.index()] {
                    res.push(
                        MeshBundle {
                        mesh : p.clone(),
                        location : location.clone(&device),
                        material : m.clone()
                    });
                }
            }
        }

        res
    }

    fn wgpu_gltf_load(&mut self, device : &wgpu::Device, path : String, world : &mut space_core::bevy::prelude::World) -> Vec<Entity> {
        let mut res = vec![];

        let sponza = gltf::Gltf::open(&path).unwrap();
        let base = PathBuf::from(&path).parent().unwrap().to_str().unwrap().to_string();

        let mut buffers = vec![];
        for buf in sponza.buffers() {
            match buf.source() {
                gltf::buffer::Source::Bin => {

                }
                gltf::buffer::Source::Uri(uri) => {

                    let mut f = std::fs::File::open(format!("{}/{}", &base, uri)).unwrap();
                    let metadata = std::fs::metadata(&format!("{}/{}", &base, uri)).unwrap();
                    let mut byte_buffer = vec![0; metadata.len() as usize];
                    f.read(&mut byte_buffer).unwrap();
                    buffers.push(byte_buffer);
                }
            }
        }

        let mut meshes = vec![];

        for m in sponza.meshes() {
            let mut combined = vec![];
            for p in m.primitives() {
                let mut pos : Vec<f32> = vec![];
                let mut normals : Vec<f32> = vec![];
                let mut uv : Vec<f32> = vec![];
                let mut tangent : Vec<f32> = vec![];

                let indices_acc = p.indices().unwrap();
                let indices_view = indices_acc.view().unwrap();
                let mut indices;

                match indices_acc.data_type() {
                    ComponentType::U16 => {
                        indices = vec![0; indices_acc.count()];
                        let buf = &buffers[indices_view.buffer().index()];
                        for idx in 0..indices.len() {
                            let global_idx = idx * indices_view.stride().unwrap_or(2) + indices_view.offset() + indices_acc.offset();
                            indices[idx] = byteorder::LittleEndian::read_u16(&buf[global_idx..(global_idx + 2)]) as u32;
                        }
                    }
                    ComponentType::U32 => {
                        indices = vec![0; indices_acc.count()];
                        let buf = &buffers[indices_view.buffer().index()];
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
                //load diffuse texture
                let vertex_count = pos.len() / 3;
                let triangle_count = indices.len() / 3;

                if uv.len() == 0 {
                    uv = vec![0.0f32; pos.len() / 3 * 2];
                }

                if tangent.len() == 0 {
                    tangent = vec![0.0f32; pos.len()];
                    println!("[ERROR] No tangents for object!");
                }



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

                let color = self.load_gltf_color_texture(&base, p.material().pbr_metallic_roughness().base_color_texture(), true);

                let mr = self.load_gltf_color_texture(&base, p.material().pbr_metallic_roughness().metallic_roughness_texture(), true);

                let material = Material {
                    color,
                    normal,
                    metallic_roughness: mr,
                    gbuffer_bind : None,
                    version_sum : 0
                };

                combined.push((GMeshPtr {mesh : Arc::new(model)}, material));
            }
            meshes.push(combined);
        }

        for n in sponza.nodes() {
            if let Some(mesh_idx) = n.mesh() {
                let mut location = Location::new(&device);

                let (tr, quat, scale) = n.transform().decomposed();
                let q = nalgebra::Quaternion::new(quat[3], quat[0], quat[1], quat[2]);
                let rot = nalgebra::Rotation::from(nalgebra::UnitQuaternion::from_quaternion(q));
                let (e_x, e_y, e_z) = rot.euler_angles();
                location.pos = tr.into();
                location.rotation = [e_x, e_y, e_z].into();
                location.scale = scale.into();

                for (p, m) in &meshes[mesh_idx.index()] {

                    let e = world.spawn(MeshBundle {
                        mesh : p.clone(),
                        location : location.clone(&device),
                        material : m.clone()
                    });
                    res.push(e.id());
                }
            }
        }

        res
    }
}

use space_core::bevy::prelude::Bundle;

#[derive(Bundle)]
pub struct MeshBundle {
    pub mesh : GMeshPtr,
    pub location : Location,
    pub material : Material
}