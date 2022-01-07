use crate::mesh::*;
use std::collections::HashMap;

pub struct SimpleWavefrontParser;

impl SimpleWavefrontParser {

    fn parse_indexses(data : &str) -> (usize, usize, usize) {
        let words = data.split("/").collect::<Vec<&str>>();

        let i1 : usize = words[0].parse().unwrap();
        let i2 : usize = words[1].parse().unwrap();
        let i3 : usize = words[2].trim().parse().unwrap();

        (i1, i2, i3)
    }

    fn save_float_parse(data : &str) -> f32 {
        data.trim().parse::<f32>().unwrap()
    }

    pub fn from_str(data : &String) -> Result<CPUMesh, String> {
        let mut mesh = CPUMesh::default();

        let mut poses : Vec<Vec3> = vec![Vec3::default()];
        let mut normals : Vec<Vec3> = vec![Vec3::default()];
        let mut tex_coords : Vec<Vec2> = vec![Vec2::default()];

        let mut index_hash : HashMap<String, u32> = HashMap::new();
        
        let mut reuse_count = 0;

        for line in data.split("\n") {
            // let line = tmp.clone().replace(".", ",");
            if line.len() > 0 {
                //its not a comment
                if line.get(0..1) != Some("#") {
                    let words = line.split(" ").collect::<Vec<&str>>();
                    match words.get(0) {
                        Some(&"o") => {
                            // println!("New object {}", words.get(1).unwrap());
                            // index_shift = mesh.verts.len();
                        }
                        Some(&"v") => {
                            let mut pos = Vec3::default();

                            pos.x = words[1].parse().unwrap();
                            pos.y = words[2].parse().unwrap();
                            pos.z = SimpleWavefrontParser::save_float_parse(words[3]);

                            // pos.z = words[3].parse().unwrap();

                            poses.push(pos);


                            // println!("{:?}", pos);
                        }
                        Some(&"vn") => { //parsing normal vector
                            let mut normal = Vec3::default();
                            normal.x = words[1].parse().unwrap();
                            normal.y = words[2].parse().unwrap();
                            normal.z = words[3].trim().parse().unwrap();

                            normals.push(normal);
                        }
                        Some(&"vt") => {
                            let mut uv = Vec2::default();
                            uv.x = words[1].parse().unwrap();
                            uv.y = words[2].trim().parse().unwrap();

                            tex_coords.push(uv);
                        }
                        Some(&"f") => {
                            for i in 1..4 {
                                if !index_hash.contains_key(words[i]) {
                                    let (pos_idx, uv_idx, normal_idx) = SimpleWavefrontParser::parse_indexses(words[i]);
                                    //alloc new vertex and add index
                                    let next_index = mesh.verts.len() as u32;
                                    index_hash.insert(String::from(words[i]), next_index);

                                    let mut vert = Vertex::default();
                                    vert.pos = poses[pos_idx];
                                    // vert.normal = poses[normal_idx];
                                    vert.tex_coord = tex_coords[uv_idx];

                                    mesh.verts.push(vert);
                                    mesh.indices.push(next_index);
                                } else {
                                    mesh.indices.push(index_hash[words[i]]);
                                    reuse_count += 1;
                                }
                            }
                        }
                        None => {

                        }
                        _ => {

                        }
                    }
                }
            }
        }

        println!("Mesh vert count: {} Triangle count: {} Reuse count: {}", mesh.verts.len(), mesh.indices.len() / 3, reuse_count);

        Ok(mesh)
    }
}