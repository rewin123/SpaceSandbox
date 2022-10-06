use std::sync::Arc;
use ash::vk;
use ash::vk::BufferUsageFlags;
use tobj::LoadError;
use crate::{BufferSafe, GPUMesh, GraphicBase};
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

        scene.push(
            Arc::new(GPUMesh {
                pos_data,
                index_data,
                normal_data,
                uv_data,
                vertex_count: mesh.indices.len() as u32,
                name : m.name.clone()
            })
        );
    }

    Ok(scene)
}