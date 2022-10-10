mod example_pipeline;
mod grayscale_pipeline;
mod single_texture_pipeline;
mod gbuffer_fill;
mod texture_demonstrate;
mod mesh_light;
mod point_light_shadow;
mod texture_transform_pipeline;

use std::sync::Arc;
use ash::vk::{CommandBuffer, ImageView};
pub use grayscale_pipeline::*;
pub use single_texture_pipeline::*;
pub use gbuffer_fill::*;
pub use texture_demonstrate::*;
pub use mesh_light::*;
pub use point_light_shadow::*;
pub use texture_transform_pipeline::*;

use crate::{ApiBase, FramebufferSafe, GraphicBase, RenderCamera, RenderServer, TextureSafe};
use crate::asset_server::AssetServer;

pub trait InstancesDrawer {
    fn process(
        &mut self,
        cmd : CommandBuffer,
        input : &[Arc<TextureSafe>],
        fb : &Arc<FramebufferSafe>,
        server : &RenderServer,
        assets : &AssetServer);
    fn create_framebuffer(&mut self) -> Arc<FramebufferSafe>;
    fn set_camera(&mut self, camera : &RenderCamera);
}

pub trait TextureTransform {
    fn process(&mut self, cmd : CommandBuffer, dst : &Vec<Arc<TextureSafe>>, input : Vec<Arc<TextureSafe>>);
    fn create_framebuffer(&mut self) -> Arc<FramebufferSafe>;
}