use std::sync::Arc;
use ash::vk;
use ash::vk::BufferUsageFlags;
use space_core::SpaceResult;
use tobj::LoadError;
use wgpu::util::DeviceExt;
use crate::{BufferSafe, GPUMesh, GraphicBase, GMesh, GVertex};
use log::*;

pub fn load_gray_obj_now(graphic_base : &GraphicBase, path : String) -> Result<Vec<Arc<GPUMesh>>, LoadError> {
    let (models, _) = tobj::load_obj(path,
                                             &tobj::GPU_LOAD_OPTIONS)?;

    let mut scene = vec![];


    for (_, m) in models.iter().enumerate() {
        info!("Found model {}!", m.name.clone());

        let mesh = &m.mesh;

        let mut chandeg_pos = vec![];
        for vertex_idx in 0..(mesh.positions.len() / 3) {
            chandeg_pos.push(mesh.positions[vertex_idx * 3]);
            chandeg_pos.push(mesh.positions[vertex_idx * 3 + 1]);
            chandeg_pos.push(mesh.positions[vertex_idx * 3 + 2]);
        }


        let mut pos_data = BufferSafe::new(
            &graphic_base.allocator,
            (chandeg_pos.len() * 4) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu
        ).unwrap();

        let mut index_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.indices.len() * 4) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu
        ).unwrap();

        let mut normal_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.normals.len() * 3) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu
        ).unwrap();

        let mut tangent_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.normals.len() * 3) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu
        ).unwrap();

        let mut uv_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.normals.len() * 4) as u64,
            BufferUsageFlags::VERTEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu
        ).unwrap();


        pos_data.fill(&chandeg_pos).unwrap();
        index_data.fill(&mesh.indices).unwrap();
        normal_data.fill(&mesh.normals).unwrap();
        uv_data.fill(&vec![0.0f32; mesh.normals.len()]).unwrap();
        tangent_data.fill(&vec![0.0f32; mesh.normals.len()]).unwrap();


        scene.push(
            Arc::new(GPUMesh {
                pos_data,
                index_data,
                normal_data,
                tangent_data: tangent_data,
                uv_data,
                vertex_count: mesh.indices.len() as u32,
                name : m.name.clone()
            })
        );
    }

    Ok(scene)
}

pub fn wgpu_load_gray_obj(device : &wgpu::Device, path : String) -> SpaceResult<Vec<Arc<GMesh>>> {
    let (models, _) = tobj::load_obj(path,
                                             &tobj::GPU_LOAD_OPTIONS)?;

    let mut scene = vec![];


    for (_, m) in models.iter().enumerate() {
        info!("Found model {}!", m.name.clone());

        let mesh = &m.mesh;

        let vertex : Vec<GVertex> = (0..(mesh.positions.len() / 3)).into_iter().map(|idx| {
            
            let shift = idx * 3;
            let uv_shift = idx * 2;
            
            GVertex {
                pos: [mesh.positions[shift], mesh.positions[shift + 1],mesh.positions[shift + 2]],
                normal: [mesh.normals[shift], mesh.normals[shift + 1],mesh.normals[shift + 2]],
                tangent: [0.0, 0.0, 0.0],
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

    Ok(scene)
}