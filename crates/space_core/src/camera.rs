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

#[derive(Default)]
pub struct Ray {
    pub pos : nalgebra::Point3<f32>,
    pub dir : nalgebra::Vector3<f32>
}

impl Ray {
    pub fn interact_y(&self, y : f32) -> na::Point3<f32> {
        let dy = y - self.pos.y;
        let t = dy / self.dir.y;
        let pos = self.pos + t * self.dir;
        pos
    }
}

impl Camera {
    pub fn get_right(&self) -> na::Vector3<f32> {
        self.frw.cross(&self.up)
    }

    pub fn screen_pos_to_ray(
        &self,
        screen_pos : nalgebra::Point2<f32>,
        screen_size : nalgebra::Point2<f32>
    ) -> Ray {
        let mut res = Ray::default();

        let uniform = self.build_uniform();

        let up = self.pos + self.up;
        let right = self.pos + self.get_right();

        let screen_up = uniform.proj * uniform.view * nalgebra::Vector4::<f32>::new(up.x, up.y, up.z, 1.0);
        let screen_right = uniform.proj * uniform.view * nalgebra::Vector4::<f32>::new(right.x, right.y, right.z, 1.0);;

        let uniform_screen_pos = nalgebra::Vector2::<f32>::new(
            screen_pos.x / screen_size.x * 2.0 - 1.0,
            screen_pos.y / screen_size.y * 2.0 - 1.0);


        let k_x = uniform_screen_pos.x / screen_right.x;
        let k_y = -uniform_screen_pos.y / screen_up.y;

        let mut dir = k_x * self.get_right() + k_y * self.up + self.frw;
        dir = dir.normalize();

        res.dir = dir;
        res.pos = self.pos;

        res
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