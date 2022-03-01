
use std::{sync::Arc, fs::File, io::{BufReader, BufRead}};
use vulkano;
use crate::math::*;

#[repr(C)]
#[derive(Default, Debug, Clone)]
pub struct Vertex {
    pub position : [f32; 3]
}
vulkano::impl_vertex!(Vertex, position);

pub struct CpuMesh {
    pub vertex : Vec<Vec3>,
    pub indices : Vec<i32>
}

pub fn mesh_from_file(path : String) -> Option<Arc<CpuMesh>> {
    wavefront_mesh_from_file(path)
}

fn wavefront_indices(data : &str) -> (i32, i32, i32) {
    let split = data.split('/').collect::<Vec<_>>();

    (split[0].parse().unwrap(), split[1].parse().unwrap(), split[2].parse().unwrap())
}

pub fn wavefront_mesh_from_file(path : String) -> Option<Arc<CpuMesh>> {

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut poses = vec![];
    let mut normals = vec![];
    // let mut uvs = vec![];
    let mut idx_shift = 0;

    for line_result in reader.lines() {
        let line = line_result.unwrap();
        let words = line.split(" ").collect::<Vec<&str>>();
        
        if words[0] == "o" {
            //new object
            idx_shift = poses.len();
        } else if words[0] == "v" {
            //new pos
            poses.push(Vec3::new(
                words[1].parse().unwrap(), 
                words[2].parse().unwrap(), 
                words[3].parse().unwrap()));
        } else if words[0] == "f" {
            
        }
    }


    None
}