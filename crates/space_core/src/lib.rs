
mod task_server;
mod camera;

pub use task_server::*;
pub use camera::*;

pub use bevy;
pub use ron;
pub use serde;

pub use bevy::ecs::prelude as ecs;
pub use bevy::app::prelude as app;
pub use bevy::asset::prelude as asset;
pub use nalgebra;



#[derive(Debug)]
pub struct RenderBase {
    pub device : wgpu::Device,
    pub queue : wgpu::Queue,
}


pub struct ScreenMesh {
    pub vertex : wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    pub pos : [f32; 3]
}

impl SimpleVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub type SpaceResult<T> = Result<T, Box<dyn std::error::Error>>;