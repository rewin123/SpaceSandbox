use downcast_rs::impl_downcast;

use crate::AssetPath;

pub mod wgpu_gbuffer_fill;
pub mod wgpu_light_fill;
pub mod wgpu_texture_present;
pub mod wgpu_light_shadow;
pub mod wgpu_textures_transform;

pub trait Pipeline {
    fn get_shader_path(&self) -> AssetPath;
    fn rebuild_with_new_shader(&mut self, shader : AssetPath, camera_buffer : &wgpu::Buffer);
}
