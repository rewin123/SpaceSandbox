use std::sync::Arc;

use vulkano::{image::view::ImageView, pipeline::{GraphicsPipeline, graphics::viewport::Viewport}, render_pass::RenderPass};

use crate::rpu::RPU;


pub struct DirectLightRender {
    pub rpu : RPU,
    pub target : Arc<dyn vulkano::image::view::ImageViewAbstract>,
    pub pipeline : Arc<GraphicsPipeline>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
}

impl DirectLightRender {
    pub fn from_rpu(rpu: RPU) -> Self {

        Self {
            rpu : rpu,
        }
    }
}