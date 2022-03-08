
use std::sync::Arc;

use cgmath::*;
use vulkano::{device::Device, buffer::{CpuBufferPool, BufferUsage, cpu_pool::*}, memory::pool::StdMemoryPool};

pub struct Camera {
    pub position : cgmath::Point3<f32>,
    pub aspect_ratio : f32
}

impl Camera {

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
                Point3::new(0.0, 0.0, 0.0),
                Vector3::new(0.0, -1.0, 0.0),
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