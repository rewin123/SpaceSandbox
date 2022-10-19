use std::fmt::Debug;
use downcast_rs::{Downcast, impl_downcast};

pub mod wgpu_gbuffer_fill;
pub mod wgpu_light_fill;
pub mod wgpu_texture_present;
pub mod wgpu_light_shadow;
pub mod wgpu_textures_transform;

use space_assets::*;

pub use wgpu_gbuffer_fill::*;
pub use wgpu_light_fill::*;
pub use wgpu_texture_present::*;
pub use wgpu_light_shadow::*;
pub use wgpu_textures_transform::*;

pub trait PipelineDesc : Downcast + Debug {
    fn get_shader_path(&self) -> AssetPath;
    fn set_shader_path(&mut self, path : AssetPath);
    fn clone_boxed(&self) -> Box<dyn PipelineDesc>;
}
impl_downcast!(PipelineDesc);

pub trait Pipeline {
    fn new_described(desc : Box<dyn PipelineDesc>, camera_buffer : &wgpu::Buffer) -> Self;
    fn get_desc(&self) -> Box<dyn PipelineDesc>;
}
