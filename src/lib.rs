use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use log::*;
use simplelog::*;
use std::default::Default;
use gpu_allocator::vulkan::{Allocation, AllocatorCreateDesc};
// use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

pub mod assets;
pub mod pipelines;
pub mod task_server;
pub mod ui;
pub mod light;

pub use assets::runtime_gpu_assets::*;
pub use assets::*;
pub use pipelines::*;

pub struct RenderBase {
    pub device : wgpu::Device,
    pub queue : wgpu::Queue
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GVertex {
    pub pos : [f32; 3],
    pub normal : [f32; 3],
    pub tangent : [f32; 3],
    pub uv : [f32; 2]
}

impl GVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 4 * 3,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 4 * 3 * 2,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 4 * 3 * 3,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub struct GMesh {
    pub vertex : wgpu::Buffer,
    pub index : wgpu::Buffer,
    pub index_count : u32
}

pub struct Material {
    pub color : ServerTexture,
    pub normal : ServerTexture,
    pub metallic_roughness: ServerTexture
}

pub fn init_logger() {
    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("detailed.log").unwrap())
        ]
    );
}
