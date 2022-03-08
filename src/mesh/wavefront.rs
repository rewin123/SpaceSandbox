use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use crate::math::*;
use crate::mesh::{CpuMesh, Vertex};

fn wavefront_indices(data : &str) -> (i32, i32, i32) {
    let split = data.split('/').collect::<Vec<_>>();

    (split[0].parse().unwrap(),
     split[1].parse().unwrap(),
     split[2].parse().unwrap())
}

fn get_vec3(words : &Vec<String>) -> Vec3 {
    Vec3::new(
        words[1].parse().unwrap(),
        words[2].parse().unwrap(),
        words[3].parse().unwrap())
}

fn get_vec2(words : &Vec<String>) -> Vec2 {
    Vec2 {
        data : [
            words[1].parse().unwrap(),
            words[2].parse().unwrap()]
    }
}

pub fn mesh_from_file(path : String) -> Option<CpuMesh> {

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut poses = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut idx_shift : i32 = 0;

    let mut verts = vec![];
    let mut indices : Vec<u32> = vec![];

    let mut idx_map = HashMap::new();

    for line_result in reader.lines() {
        let line = line_result.unwrap();
        if line.len() == 0 {
            continue;
        }
        let words_str = line.split(" ").collect::<Vec<&str>>();

        let mut words = vec![];
        for i in 0..words_str.len() {
            words.push(String::from(words_str[i]));
        }

        if words[0] == "o" {
            //new object
            idx_shift = poses.len() as i32;
        } else if words[0] == "v" {
            //new pos
            poses.push(get_vec3(&words));
        } else if words[0] == "vn" {
            normals.push(get_vec3(&words));
        } else if words[0] == "vt" {
            uvs.push(get_vec2(&words));
        } else if words[0] == "f" {
            for i in 1..4 {
                if !idx_map.contains_key(&words[i]) {
                    let (pos_idx, uv_idx, norm_idx) = wavefront_indices(&words[i]);

                    let vert = Vertex {
                        position : poses[(pos_idx - 1) as usize].data,
                        normal : normals[(norm_idx - 1) as usize].data
                    };
                    verts.push(vert);

                    // vertex_poses.push(poses[(pos_idx - 1) as usize]);
                    // vertex_normals.push(normals[(norm_idx - 1) as usize]);
                    // vertex_uvs.push(uvs[(uv_idx - 1) as usize]);
                    idx_map.insert(
                        words[i].clone(),
                        verts.len() - 1
                    );
                }
                indices.push(idx_map[&words[i]] as u32);
            }
        }
    }

    let mesh = CpuMesh {
        verts,
        indices
    };

    Some(mesh)
}

#[cfg(test)]
mod WavefrontTest {
    use crate::mesh::wavefront::mesh_from_file;

    #[test]
    fn test_wavefront_loading() {
        let mesh = mesh_from_file(
            String::from("res/test/wavefront/test.obj")).unwrap();
        println!("{:?}",mesh);
        assert_eq!(mesh.indices.len(), 3);
        assert_eq!(mesh.verts.len(), 1);
        assert_eq!(mesh.verts[0].position[0], 1.0);
    }

}