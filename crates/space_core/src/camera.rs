use encase::*;
use nalgebra as na;
use crate::ecs::*;

#[derive(ShaderType)]
pub struct CameraUniform {
    pub view : nalgebra::Matrix4<f32>,
    pub proj : nalgebra::Matrix4<f32>,
    pub pos : nalgebra::Vector3<f32>
}

#[derive(Clone, Resource)]
pub struct Camera {
    pub pos : nalgebra::Point3<f32>,
    pub frw : nalgebra::Vector3<f32>,
    pub up : nalgebra::Vector3<f32>
}

impl Camera {
    pub fn get_right(&self) -> na::Vector3<f32> {
        self.frw.cross(&self.up)
    }
}



impl Default for Camera {
    fn default() -> Self {
        Self {
            pos : [-3.0, 9.0, 0.0].into(),
            frw : [1.0, 0.0, 0.0].into(),
            up : [0.0, 1.0, 0.0].into()
        }
    }
}

impl Camera {
    pub fn build_uniform(&self) -> CameraUniform {

        let mut target = self.pos + self.frw;
        let view = nalgebra::Matrix4::look_at_rh(
            &self.pos,
            &target,
            &self.up);
        let proj = nalgebra::Matrix4::<f32>::new_perspective(
            1.0,
            3.14 / 2.0,
            0.01,
            10000.0);
        CameraUniform {
            view,
            proj,
            pos : na::Vector3::new(self.pos.x, self.pos.y, self.pos.z)
        }
    }
}