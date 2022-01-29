
use std::time::Instant;
use std::iter;

use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use epi::*;
use winit::window::Window;
use std::sync::Arc;

struct RepaintSignalStruct;

impl epi::backend::RepaintSignal for RepaintSignalStruct {
    fn request_repaint(&self) {
    }
}

pub struct GUIRender {
    pub platform : Platform,
    start_time : Instant,
    repaint_signal : Arc<RepaintSignalStruct>,
    scale_factor : f64,
    egui_start : Instant,
    previous_frame_time  : Option<f32>,
    egui_rpass : RenderPass
}

impl GUIRender {

    pub fn new(window : &Window, gpu : &crate::gpu::GPU) -> Self {

        let repaint_signal = std::sync::Arc::new(RepaintSignalStruct{});

        let size = window.inner_size();

        let mut platform = Platform::new(PlatformDescriptor {
            physical_width: size.width as u32,
            physical_height: size.height as u32,
            scale_factor: window.scale_factor(),
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let surface_format = gpu.surface.get_preferred_format(&gpu.adapter).unwrap();
        let mut egui_rpass = RenderPass::new(&gpu.device, surface_format, 1);

        Self {
            platform,
            start_time: Instant::now(),
            repaint_signal,
            scale_factor : window.scale_factor(),
            previous_frame_time : None,
            egui_start : Instant::now(),
            egui_rpass
        }
    }

    pub fn start_gui_draw(&mut self) {
        self.platform.update_time(self.start_time.elapsed().as_secs_f64());
        self.platform.begin_frame();
        
        self.egui_start = Instant::now();
        let app_output = epi::backend::AppOutput::default();

        epi::Frame::new(epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "egui_example",
                web_info: None,
                cpu_usage: self.previous_frame_time,
                native_pixels_per_point: Some(self.scale_factor as _),
                prefer_dark_mode: Some(true),
            },
            output: app_output,
            repaint_signal: self.repaint_signal.clone(),
        });
        

    }

    pub fn end_gui_draw(&mut self, gpu : &crate::gpu::GPU, frame_result : &wgpu::SurfaceTexture) {
        let (_output, paint_commands) = self.platform.end_frame(None);
        let paint_jobs = self.platform.context().tessellate(paint_commands);

        let frame_time = (Instant::now() - self.egui_start).as_secs_f64() as f32;
        self.previous_frame_time = Some(frame_time);

        let mut encoder = gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("encoder"),
        });

        let output_view = frame_result
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

        // Upload all resources for the GPU.
        let screen_descriptor = ScreenDescriptor {
            physical_width: gpu.surface_config.width,
            physical_height: gpu.surface_config.height,
            scale_factor: self.scale_factor as f32,
        };
        self.egui_rpass.update_texture(&gpu.device, &gpu.queue, &self.platform.context().font_image());
        self.egui_rpass.update_user_textures(&gpu.device, &gpu.queue);
        self.egui_rpass.update_buffers(&gpu.device, &gpu.queue, &paint_jobs, &screen_descriptor);

        // Record all render passes.
        self.egui_rpass
            .execute(
                &mut encoder,
                &output_view,
                &paint_jobs,
                &screen_descriptor,
                Some(wgpu::Color::BLACK),
            )
            .unwrap();
        // Submit the commands.
        gpu.queue.submit(iter::once(encoder.finish()));
    }
}