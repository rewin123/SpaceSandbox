use bytemuck::{Pod, Zeroable};


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vec3 {
    x : f32,
    y : f32,
    z : f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vec2 {
    x : f32,
    y : f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pos: Vec3,
    tex_coord: Vec2,
}


pub fn create_cube_vertices() -> (Vec<Vertex>, Vec<u32>) {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::from_pos_uv([-1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::from_pos_uv([1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::from_pos_uv([1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::from_pos_uv([-1.0, 1.0, 1.0], [0.0, 1.0]),
        // bottom (0, 0, -1)
        Vertex::from_pos_uv([-1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::from_pos_uv([1.0, 1.0, -1.0], [0.0, 0.0]),
        Vertex::from_pos_uv([1.0, -1.0, -1.0], [0.0, 1.0]),
        Vertex::from_pos_uv([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // right (1.0, 0, 0)
        Vertex::from_pos_uv([1.0, -1.0, -1.0], [0.0, 0.0]),
        Vertex::from_pos_uv([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::from_pos_uv([1.0, 1.0, 1.0], [1.0, 1.0]),
        Vertex::from_pos_uv([1.0, -1.0, 1.0], [0.0, 1.0]),
        // left (-1.0, 0, 0)
        Vertex::from_pos_uv([-1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::from_pos_uv([-1.0, 1.0, 1.0], [0.0, 0.0]),
        Vertex::from_pos_uv([-1.0, 1.0, -1.0], [0.0, 1.0]),
        Vertex::from_pos_uv([-1.0, -1.0, -1.0], [1.0, 1.0]),
        // front (0, 1.0, 0)
        Vertex::from_pos_uv([1.0, 1.0, -1.0], [1.0, 0.0]),
        Vertex::from_pos_uv([-1.0, 1.0, -1.0], [0.0, 0.0]),
        Vertex::from_pos_uv([-1.0, 1.0, 1.0], [0.0, 1.0]),
        Vertex::from_pos_uv([1.0, 1.0, 1.0], [1.0, 1.0]),
        // back (0, -1.0, 0)
        Vertex::from_pos_uv([1.0, -1.0, 1.0], [0.0, 0.0]),
        Vertex::from_pos_uv([-1.0, -1.0, 1.0], [1.0, 0.0]),
        Vertex::from_pos_uv([-1.0, -1.0, -1.0], [1.0, 1.0]),
        Vertex::from_pos_uv([1.0, -1.0, -1.0], [0.0, 1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

impl Vertex {
    pub fn from_pos_uv(pos: [f32; 3], uv: [f32; 2]) -> Vertex {
        Vertex {
            pos : Vec3{x : pos[0], y : pos[1], z : pos[2]},
            tex_coord : Vec2 {x : uv[0], y : uv[1]}
        }
    }
}

pub struct CPUMesh {
    pub verts : Vec<Vertex>,
    pub indices : Vec<u32>
}

pub struct GPUMesh {
    vertex_buffer : wgpu::Buffer,
    index_buffer : wgpu::Buffer
}