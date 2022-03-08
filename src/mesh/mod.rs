use std::{sync::Arc, fs::File, io::{BufReader, BufRead}};
use std::collections::HashMap;
use vulkano;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::device::Device;
use crate::math::*;

pub mod wavefront;

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position : [f32; 3],
    pub normal : [f32; 3]
}
vulkano::impl_vertex!(Vertex, position, normal);

#[derive(Debug)]
pub struct CpuMesh {
    pub verts : Vec<Vertex>,
    pub indices : Vec<u32>
}

pub struct GpuMesh {
    pub verts : Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub indices : Arc<CpuAccessibleBuffer<[u32]>>,
}

impl CpuMesh {
    pub fn scale(&mut self, factor : f32) {
        for i in 0..self.verts.len() {
            let mut pos = self.verts[i].position;
            pos[0] *= factor;
            pos[1] *= factor;
            pos[2] *= factor;
            self.verts.get_mut(i).unwrap().position = pos;
        }
    }
}

impl GpuMesh {
    pub fn from_cpu(mesh : Arc<CpuMesh>, device : Arc<Device>) -> Arc<GpuMesh> {
        let verts = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            mesh.verts.iter().cloned()).unwrap();

        let indices = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            mesh.indices.clone()).unwrap();

        Arc::new(
            GpuMesh {
                verts,
                indices
            }
        )
    }

    pub fn from_file(device : Arc<Device>, path : String) -> Option<Arc<GpuMesh>> {
        match mesh_from_file(path) {
            Some(cpu_mesh ) => {
                Some(GpuMesh::from_cpu(Arc::new(cpu_mesh), device))
            }
            None => {
                None
            }
        }
    }
}

pub fn mesh_from_file(path : String) -> Option<CpuMesh> {
    wavefront::mesh_from_file(path)
}


#[cfg(test)]
mod mesh_tests {
    use crate::math::{Vec2, Vec3};
    use crate::mesh::*;
    use crate::rpu::RPU;

    #[test]
    fn cpu_to_gpu() {
        let rpu = RPU::default();
        let cpu_mesh = Arc::new(CpuMesh {
            verts : vec![
                Vertex {
                    position : [0.0, 0.0, 0.0], 
                    normal : [0.0, 0.0, 0.0]
                }
            ],
            indices: vec![0]
        });

        let gpu_mesh = GpuMesh::from_cpu(
            cpu_mesh, rpu.device.clone());
    }
}




