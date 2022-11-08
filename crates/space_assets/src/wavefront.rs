use std::sync::Arc;
use space_core::SpaceResult;
use tobj::LoadError;
use wgpu::util::DeviceExt;
use crate::GMeshPtr;
use crate::mesh::{GMesh, GVertex};


pub fn wgpu_load_gray_obj(device : &wgpu::Device, path : String) -> SpaceResult<Vec<GMeshPtr>> {
    let (models, _) = tobj::load_obj(path,
                                             &tobj::GPU_LOAD_OPTIONS)?;

    let mut scene = vec![];


    for (_, m) in models.iter().enumerate() {

        let mesh = &m.mesh;

        let vertex : Vec<GVertex> = (0..(mesh.positions.len() / 3)).into_iter().map(|idx| {
            
            let shift = idx * 3;
            let uv_shift = idx * 2;
            
            GVertex {
                pos: [mesh.positions[shift], mesh.positions[shift + 1],mesh.positions[shift + 2]],
                normal: [mesh.normals[shift], mesh.normals[shift + 1],mesh.normals[shift + 2]],
                tangent: [1.0, 0.0, 0.0],
                uv: [mesh.texcoords[uv_shift], mesh.texcoords[uv_shift + 1]],
            }
        }).collect();

        let vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("obj vertex"),
            contents: bytemuck::cast_slice(&vertex),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("onj index"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });


        scene.push(
            Arc::new(GMesh { vertex, index, index_count: mesh.indices.len() as u32 })
        );
    }

    Ok(scene.iter().map(|mesh| GMeshPtr {mesh : mesh.clone()}).collect())
}