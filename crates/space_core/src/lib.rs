
mod task_server;

pub use task_server::*;


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