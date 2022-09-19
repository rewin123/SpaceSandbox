use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use ash::vk;
use nalgebra as na;
use crate::{AllocatorSafe, BufferSafe};

pub struct RenderCamera {
    pub camera : Camera,
    pub uniformbuffer : BufferSafe
}

impl RenderCamera {
    pub fn new(allocator : &Arc<AllocatorSafe>) -> Self {
        let mut camera = Camera::default();

        let mut uniformbuffer = BufferSafe::new(
            &allocator,
            64 * 2,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu
        ).unwrap();
        let cameratransform: [[[f32; 4]; 4]; 2] = [
            nalgebra::Matrix4::identity().into(),
            nalgebra::Matrix4::identity().into()
        ];
        uniformbuffer.fill(&cameratransform).unwrap();

        Self {
            camera,
            uniformbuffer
        }
    }

    pub fn update_inner_buffer(&mut self) {
        self.camera.update_buffer(&mut self.uniformbuffer);
    }
}

impl Deref for RenderCamera {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.camera
    }
}

impl DerefMut for RenderCamera {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.camera
    }
}

pub struct Camera {
    viewmatrix : na::Matrix4<f32>,
    pub position: na::Vector3<f32>,
    pub view_direction: na::Vector3<f32>,
    pub down_direction: na::Vector3<f32>,
    pub fovy: f32,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
    projectionmatrix: na::Matrix4<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        let mut camera = Camera {
            viewmatrix: na::Matrix4::identity(),
            position: na::Vector3::new(0.0, 25.0, -100.0),
            view_direction: na::Vector3::new(0.0, 0.0, 1.0),
            down_direction: na::Vector3::new(0.0, -1.0, 0.0),
            fovy : std::f32::consts::FRAC_PI_3,
            aspect : 800.0 / 600.0,
            near : 0.1,
            far : 1000.0,
            projectionmatrix : na::Matrix4::identity()
        };
        camera.update_viewmatrix();
        camera.update_projectionmatrix();
        camera
    }
}

impl Camera {

    pub fn update_projectionmatrix(&mut self) {
        let d = 1.0 / (0.5 * self.fovy).tan();
        self.projectionmatrix = na::Matrix4::new(
            d / self.aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            d,
            0.0,
            0.0,
            0.0,
            0.0,
            self.far / (self.far - self.near),
            -self.near * self.far / (self.far - self.near),
            0.0,
            0.0,
            1.0,
            0.0,
        );
    }

    pub fn update_buffer(&self, buffer: &mut BufferSafe) {
        let data: [[[f32; 4]; 4]; 2] = [self.viewmatrix.into(), self.projectionmatrix.into()];
        buffer.fill(&data);
    }
    pub fn update_viewmatrix(&mut self) {
        let right = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction));
        let m = na::Matrix4::new(
            right.x,
            right.y,
            right.z,
            -right.dot(&self.position), //
            self.down_direction.x,
            self.down_direction.y,
            self.down_direction.z,
            -self.down_direction.dot(&self.position), //
            self.view_direction.x,
            self.view_direction.y,
            self.view_direction.z,
            -self.view_direction.dot(&self.position), //
            0.0,
            0.0,
            0.0,
            1.0,
        );
        self.viewmatrix =  m;
    }
    pub fn move_forward(&mut self, distance: f32) {
        self.position += distance * self.view_direction;
        self.update_viewmatrix();
    }
    pub fn move_backward(&mut self, distance: f32) {
        self.move_forward(-distance);
    }
    pub fn turn_right(&mut self, angle: f32) {
        let rotation = na::Rotation3::from_axis_angle(&na::Unit::new_normalize(self.down_direction), angle);
        self.view_direction = rotation * self.view_direction;
        self.update_viewmatrix();
    }
    pub fn turn_left(&mut self, angle: f32) {
        self.turn_right(-angle);
    }
    pub fn turn_up(&mut self, angle: f32) {
        let right = na::Unit::new_normalize(self.down_direction.cross(&self.view_direction));
        let rotation = na::Rotation3::from_axis_angle(&right, angle);
        self.view_direction = rotation * self.view_direction;
        self.down_direction = rotation * self.down_direction;
        self.update_viewmatrix();
    }
    pub fn turn_down(&mut self, angle: f32) {
        self.turn_up(-angle);
    }

    pub fn get_right_vector(&self) -> na::Vector3<f32> {
        self.down_direction.cross(&self.view_direction)
    }
}