use gltf::Node;
use specs::*;
use specs::prelude::*;
use crate::render::GMesh;

pub fn generate_static_world() -> World {
    let mut world = World::new();

    world.register::<crate::game_object::Pos>();
    world.register::<GMesh>();

    world
}


pub fn from_gltf(path : &str) -> World {
    let mut world = generate_static_world();

    let (gltf, buffers, images) = gltf::import(path).unwrap();

    for mesh in gltf.meshes() {
        println!("Mesh #{}", mesh.index());
        for primitive in mesh.primitives() {
            primitive.;
            println!("- Primitive #{}", primitive.index());
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            if let Some(iter) = reader.read_positions() {
                for vertex_position in iter {
                    println!("{:?}", vertex_position);
                }
            }
        }
    }

    world
}