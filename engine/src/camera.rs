use bytemuck::{Pod, Zeroable};

use crate::mesh::Vec4;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub pos : Vec4,
    pub frw : Vec4,
    pub up : Vec4
}

pub struct Camera {
    pub uniform : CameraUniform
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            uniform : CameraUniform {
                pos : Vec4::default(),
                frw : Vec4 {x : 1.0, y : 0.0, z : 0.0, w : 1.0},
                up : Vec4 {x : 0.0, y : 0.0, z : 1.0, w : 1.0}
            }
        }
    }
}