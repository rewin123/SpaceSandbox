mod fps_counter;
mod api_info;
mod gltf_to_load;

use std::{sync::Arc, time::Instant};

use egui::{FontDefinitions, Style, Window, FullOutput};
use egui_wgpu_backend::ScreenDescriptor;
pub use fps_counter::*;
pub use api_info::*;
pub use gltf_to_load::*;
use space_core::RenderBase;

pub struct Gui {
    pub render_pass : egui_wgpu_backend::RenderPass,
    pub platform : egui_winit_platform::Platform,
    pub start_time : Instant,
    pub render : Arc<RenderBase>
}

impl Gui {
    pub fn new(
            render : &Arc<RenderBase>, 
            format : wgpu::TextureFormat,
            size : wgpu::Extent3d,
            scale : f64) -> Self {
        let render_pass = egui_wgpu_backend::RenderPass::new(
            &render.device, 
            format, 
            1);

        let mut platform = egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: scale,
            font_definitions: FontDefinitions::default(),
            style: Style::default(),
        });

        
        Self {
            render_pass,
            platform,
            start_time : Instant::now(),
            render : render.clone()
        }
    }

    pub fn begin_frame(&mut self) {
        self.platform.update_time(
            self.start_time.elapsed().as_secs_f64());
        self.platform.begin_frame();
    }

    pub fn end_frame(&mut self, window : Option<&winit::window::Window>) -> FullOutput {
        let output = self.platform.end_frame(window);
        output
    }

    pub fn draw(&mut self, 
            output : FullOutput, 
            desc : ScreenDescriptor, 
            encoder : &mut wgpu::CommandEncoder, 
            dst : &wgpu::TextureView) {
        let paint_jobs = self.platform.context().tessellate(output.shapes);
        let tdelta = output.textures_delta;
        self.render_pass
            .add_textures(&self.render.device, &self.render.queue, &tdelta)
            .expect("ui add texture");
        self.render_pass.update_buffers(
            &self.render.device, 
            &self.render.queue, 
            &paint_jobs, 
            &desc);

        self.render_pass.execute(
            encoder, 
            dst, 
            &paint_jobs, 
            &desc, 
        None).unwrap();
    }


}