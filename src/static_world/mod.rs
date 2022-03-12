use std::sync::Arc;

use gltf::Node;
use specs::*;
use specs::prelude::*;
use vulkano::device::Device;
use crate::render::GMesh;
use crate::game_object::Pos;

pub fn generate_static_world() -> World {
    let mut world = World::new();

    world.register::<Pos>();
    world.register::<GMesh>();

    world
}


pub fn from_gltf(path : &str, device : Arc<Device>) -> World {
    let mut world = generate_static_world();

    let scenes = easy_gltf::load(path).unwrap();

    for scene in scenes {
        for model in scene.models {
            let vertices = model.vertices();
            let indices = model.indices().unwrap();

            let mut cpu_mesh = crate::mesh::CpuMesh {
                verts : vec![],
                indices : vec![],
            };

            for v in vertices {
                let vert = crate::mesh::Vertex {
                    position : v.position.into(),
                    normal : v.normal.into(),
                    uv : v.tex_coords.into()
                };

                cpu_mesh.verts.push(vert);
            }

            for idx in indices {
                cpu_mesh.indices.push(*idx as u32);
            }

            //println!("Create mesh with {} verts", cpu_mesh.verts.len());

            let gpu_mesh = crate::mesh::GpuMesh::from_cpu(Arc::new(cpu_mesh), device.clone());

            let mut gmesh = GMesh {
                mesh : gpu_mesh
            };

            world.create_entity().with(Pos(cgmath::Vector3::new(0.0, 0.0, 0.0))).with(gmesh).build();
        }
    }

    world
}