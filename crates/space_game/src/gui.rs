use std::sync::Arc;
use std::time::Instant;
use egui::{FontDefinitions, FullOutput, Style};
use egui_wgpu_backend::ScreenDescriptor;
use space_core::{RenderBase, ecs::*};
use wgpu::{TextureView, SurfaceTexture};
use space_core::ecs::*;

use crate::GlobalStageStep;

struct EguiRenderCmds {
    output : egui::FullOutput
}

fn start_gui_frame(
    mut gui : ResMut<Gui>) {
    gui.begin_frame();
}

fn end_gui_frame(
    mut gui : ResMut<Gui>,
    window : Res<winit::window::Window>,
    mut egui_cmds : ResMut<EguiRenderCmds>) {
    // egui_cmds.output = gui.end_frame(Some(window.as_ref()));
}

#[derive(Component)]
pub struct RenderTarget {
    pub view : TextureView,
    pub output : SurfaceTexture
}

fn egui_draw(
    mut gui : ResMut<Gui>,
    window : Res<winit::window::Window>,
    mut encoder : ResMut<wgpu::CommandEncoder>,
    render_target : Res<RenderTarget>,
    output : Res<EguiRenderCmds>
) {
    // gui.draw(output.output.clone(),
    //     egui_wgpu_backend::ScreenDescriptor {
    //         physical_width: window.as_ref().inner_size().width,
    //         physical_height: window.inner_size().height,
    //         scale_factor: window.scale_factor() as f32,
    //     },
    //     &mut encoder,
    //     &render_target.view);
}

pub fn setup_gui(world : &mut World, schedule : &mut Schedule) {
    
    world.insert_resource(EguiRenderCmds {output : egui::FullOutput::default()});
    
    schedule.add_system_to_stage(GlobalStageStep::RenderPrepare, start_gui_frame);
    schedule.add_system_to_stage(GlobalStageStep::PostUpdate, end_gui_frame);
    schedule.add_system_to_stage(GlobalStageStep::Render, egui_draw);
}

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