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
        self.frw.cross(&self.up).normalize()
    }

    fn pseudo_inverse(&self, mat : nalgebra::Matrix4<f32>) -> nalgebra::Matrix4<f32> {
        let mat_sopr = mat.transpose();
        let pseudo_inverse = (mat_sopr * mat).try_inverse().unwrap() * mat_sopr;
        pseudo_inverse
    }

    pub fn screen_pos_to_ray(
        &self,
        screen_pos : nalgebra::Point2<f32>,
        screen_size : nalgebra::Point2<f32>
    ) -> Ray {
        let mut res = Ray::default();
        let uniform_screen_pos = nalgebra::Vector2::<f32>::new(
            screen_pos.x / screen_size.x * 2.0 - 1.0,
            screen_pos.y / screen_size.y * 2.0 - 1.0);

        let uniform = self.build_uniform();

        let up = self.pos + self.up;
        let right = self.pos + self.get_right();
        let frw = self.pos + self.frw;

        let filter_mask = nalgebra::Matrix4::<f32>::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 0.0,
            0.0, 0.0, 0.0, 1.0);

        let w_z = 1.0f32;
        let P = uniform.proj.clone_owned();
        let s = uniform_screen_pos.clone_owned();
        let w_x = (P.m12*P.m23 + P.m12*P.m24 - P.m12*P.m43*s.y - P.m12*P.m44*s.y - P.m13*P.m22 + P.m13*P.m42*s.y - P.m14*P.m22 + P.m14*P.m42*s.y + P.m22*P.m43*s.x + P.m22*P.m44*s.x - P.m23*P.m42*s.x - P.m24*P.m42*s.x)
            /(P.m11*P.m22 - P.m11*P.m42*s.y - P.m12*P.m21 + P.m12*P.m41*s.y + P.m21*P.m42*s.x - P.m22*P.m41*s.x);
        let w_y = (-P.m11*P.m23 - P.m11*P.m24 + P.m11*P.m43*s.y + P.m11*P.m44*s.y + P.m13*P.m21 - P.m13*P.m41*s.y + P.m14*P.m21 - P.m14*P.m41*s.y - P.m21*P.m43*s.x - P.m21*P.m44*s.x + P.m23*P.m41*s.x + P.m24*P.m41*s.x)
            /(P.m11*P.m22 - P.m11*P.m42*s.y - P.m12*P.m21 + P.m12*P.m41*s.y + P.m21*P.m42*s.x - P.m22*P.m41*s.x);

        // println!("{:?}", w_x);

        let w = -w_x * self.get_right() + w_y * self.up + w_z * self.frw;

        // println!("{:?} {:?}", w, mat * w);
        //
        // let screen_up = uniform.proj * uniform.view * nalgebra::Vector4::<f32>::new(up.x, up.y, up.z, 1.0);
        // let screen_right = uniform.proj * uniform.view * nalgebra::Vector4::<f32>::new(right.x, right.y, right.z, 1.0);
        // let screen_frw = uniform.proj * uniform.view * nalgebra::Vector4::<f32>::new(frw.x, frw.y, frw.z, 1.0);
        //
        // // println!("{:?} {:?} {:?}", screen_right, screen_up, screen_frw);
        //
        //
        //
        // let k_x = uniform_screen_pos.x / screen_right.x * screen_frw.w;
        // let k_y = -uniform_screen_pos.y / screen_up.y * screen_frw.w;

        // let mut dir = k_x * self.get_right() + k_y * self.up + self.frw;
        let mut dir = nalgebra::Vector3::<f32>::new(w.x, w.y, w.z);
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