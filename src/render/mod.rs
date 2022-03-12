
use std::sync::Arc;

use cgmath::*;
use specs::{Component, VecStorage, World, WorldExt, Join};
use vulkano::{device::Device, buffer::{CpuBufferPool, BufferUsage, cpu_pool::*}, memory::pool::StdMemoryPool, image::{view::ImageView, StorageImage}, format::{Format, ClearValue}};
use crate::{mesh::GpuMesh, rpu::RPU, game_object::Pos};

pub struct Camera {
    pub position : cgmath::Point3<f32>,
    pub forward : cgmath::Vector3<f32>,
    pub up : cgmath::Vector3<f32>,
    pub aspect_ratio : f32
}

pub struct GMesh {
    pub mesh: Arc<GpuMesh>
}

pub trait Render {

}

pub struct GRender {
    pub rpu : RPU,
    pub diffuse_img : Arc<StorageImage>,
}

impl GRender {
    pub fn from_rpu(rpu : RPU, w : u32, h : u32) -> Self {
        
        let diffuse_img = rpu.create_image(w, h, Format::R8G8B8A8_SRGB).unwrap();

        let vs = standart_vertex::load(rpu.device.clone()).unwrap();
        let fs = gmesh_fragment::load(rpu.device.clone()).unwrap();

        Self {
            diffuse_img,
            rpu : rpu.clone()
        }
    }

    pub fn draw(&mut self, world : &World) {
        
        let read_mesh = world.read_storage::<GMesh>();
        let read_pos = world.read_storage::<Pos>();

        //clean image
        self.rpu.clear_image(self.diffuse_img.clone(), ClearValue::Float([0.0,0.0,0.0,0.0]));

        let diffuse_view = ImageView::new(self.diffuse_img.clone());

        for (pos, mesh) in (&read_pos, &read_mesh).join() {
            //do draw stuff
        }
    }
}

impl Component for GMesh {
    type Storage = VecStorage<Self>;
}

impl Camera {

    pub fn get_right(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::cross(self.forward, self.up).normalize()
    }

    pub fn rotate_camera(&mut self, dx : f32, dy : f32) {
        let right = self.get_right();
        self.forward = self.forward + dy * self.up;
        self.forward = cgmath::Vector3::normalize(self.forward);
        // self.up = right.cross(self.forward).normalize();
        let right = self.get_right();
        self.forward = self.forward + dx * right;
        self.forward = cgmath::Vector3::normalize(self.forward);
        
    }

    pub fn get_uniform_buffer(&self, device : Arc<Device>) -> CpuBufferPool<standart_vertex::ty::Data> {
        CpuBufferPool::<standart_vertex::ty::Data>::new(device.clone(), BufferUsage::all())
    }

    pub fn get_subbuffer(
        &self, 
        uniform_buffer : &mut CpuBufferPool<standart_vertex::ty::Data>)
            -> Arc<CpuBufferPoolSubbuffer<standart_vertex::ty::Data, Arc<StdMemoryPool>>> {
        let uniform_buffer_subbuffer = {

            let proj = cgmath::perspective(
                Rad(std::f32::consts::FRAC_PI_2),
                self.aspect_ratio,
                0.01,
                100.0,
            );
            let view = Matrix4::look_at_rh(
                self.position.clone(),
                self.position.clone() + self.forward.clone(),
                self.up.clone(),
            );
            let scale = Matrix4::from_scale(1.0);

            let uniform_data = standart_vertex::ty::Data {
                world: Matrix4::one().into(),
                view: (view * scale).into(),
                proj: proj.into(),
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        uniform_buffer_subbuffer
    }
}

pub mod standart_vertex {
    vulkano_shaders::shader!{
        ty: "vertex",
        path : "src/render/standart_vertex.glsl" ,
    }
}

pub mod gmesh_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/gmesh_fragment.glsl" ,
    }
}