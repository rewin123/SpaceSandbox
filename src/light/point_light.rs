use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use space_shaders::{PointLightUniform, ShaderUniform};
use nalgebra as na;
use wgpu::util::DeviceExt;
use winit::event::VirtualKeyCode::Mute;
use crate::RenderBase;


pub struct PointLight {
    inner : PointLightUniform,
    pub buffer : Arc<wgpu::Buffer>
}

impl PointLight {
    pub fn new(
        device : &wgpu::Device,
        position : na::Vector3<f32>,
    ) -> Self {

        let inner = PointLightUniform {
            pos: position,
            color: [1.0, 1.0, 1.0].into(),
            intensity: 10.0,
        };
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light buffer"),
            contents: &inner.get_bytes().unwrap(),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::MAP_WRITE,
        });
        
        Self { 
            inner,
            buffer : Arc::new(buffer)
        }
    }

    pub fn update_buffer(&self, render : &RenderBase) {
        let inned_data = self.inner.get_bytes().unwrap();
        let buffer = self.buffer.clone();
        self.buffer.slice(..).map_async(wgpu::MapMode::Write, move|_|{
            buffer.slice(..).get_mapped_range_mut().copy_from_slice(&inned_data);
            buffer.unmap();
        });

        // while *(update_flag.lock().unwrap()) == false {
        //     render.device.poll(wgpu::Maintain::Wait);
        //     std::thread::sleep(Duration::from_micros(10));
        // }

    }
}

impl Deref for PointLight {
    type Target = PointLightUniform;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PointLight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

