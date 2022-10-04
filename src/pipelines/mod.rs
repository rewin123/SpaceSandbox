mod example_pipeline;
mod grayscale_pipeline;
mod single_texture_pipeline;
mod gbuffer_fill;
mod texture_demonstrate;

use std::sync::Arc;
use ash::vk::{CommandBuffer, ImageView};
pub use grayscale_pipeline::*;
pub use single_texture_pipeline::*;
pub use gbuffer_fill::*;
pub use texture_demonstrate::*;
use crate::{ApiBase, GraphicBase, RenderServer, TextureSafe};
use crate::asset_server::AssetServer;

pub trait InstancesDrawer {
    fn process(&mut self, cmd : CommandBuffer, dst : &Vec<Arc<TextureSafe>>, server : &RenderServer, assets : &AssetServer);
    fn get_output_count(&self) -> usize;
}

pub trait TextureTransform {
    fn process(&mut self, cmd : CommandBuffer, dst : &Vec<Arc<TextureSafe>>, input : Vec<Arc<TextureSafe>>);
    fn get_output_count(&self) -> usize;
}