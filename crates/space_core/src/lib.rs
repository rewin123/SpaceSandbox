
mod task_server;

pub use task_server::*;


#[derive(Debug)]
pub struct RenderBase {
    pub device : wgpu::Device,
    pub queue : wgpu::Queue
}


pub type SpaceResult<T> = Result<T, Box<dyn std::error::Error>>;