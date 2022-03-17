// Copyright (c) 2021 Okko Hakola
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::sync::Arc;

use egui::{ScrollArea, TextEdit, TextStyle, Vec2};
use egui_winit_vulkano::Gui;
use vulkano::{
    device::{physical::PhysicalDevice, Device, DeviceExtensions, Features, Queue},
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceExtensions},
    swapchain,
    swapchain::{
        AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
        Swapchain, SwapchainCreationError,
    },
    sync,
    sync::{FlushError, GpuFuture},
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use SpaceSandbox::gui::SimpleGuiRenderer;
use SpaceSandbox::io::AssetLoader;
use SpaceSandbox::rpu::WinRpu;

use specs::*;
use specs::prelude::*;

#[derive(Debug, Default)]
struct Pos {
    x : f32,
    y : f32
}

pub fn main() {
    let asset = AssetLoader::new("");

    // Winit event loop & our time tracking initialization
    let (win_rpu, event_loop) = WinRpu::default();

    let mut world = SpaceSandbox::static_world::from_gltf(
        "res/test_res/models/sponza/glTF/Sponza.gltf", win_rpu.rpu.clone());

    let mut render = SpaceSandbox::render::GRender::from_rpu(win_rpu.rpu.clone(), 512, 512);

    let mut camera = SpaceSandbox::render::Camera {
        position: [5.0, 2.0, 0.0].into(),
        forward: [-1.0, 0.0, 0.0].into(),
        up: [0.0, -1.0, 0.0].into(),
        aspect_ratio: 1.0,
    };
    
    // Create renderer for our scene & ui
    let window_size = [1280, 720];
    let mut renderer =
        SimpleGuiRenderer::new(win_rpu.clone(), window_size, PresentMode::Immediate, "Minimal");
    // After creating the renderer (window, gfx_queue) create out gui integration
    let mut gui = Gui::new(renderer.surface(), renderer.queue(), false);
    // Create gui state (pass anything your state requires)
    let tex_id = gui.register_user_image_view(ImageView::new(render.cam_pos_img.clone()).unwrap());
    event_loop.run(move |event, _, control_flow| {
        // Update Egui integration so the UI works!
        gui.update(&event);
        match event {
            Event::WindowEvent { event, window_id } if window_id == window_id => match event {
                WindowEvent::Resized(_) => {
                    renderer.resize();
                }
                WindowEvent::ScaleFactorChanged { .. } => {
                    renderer.resize();
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => (),
            },
            Event::RedrawRequested(window_id) if window_id == window_id => {

                render.draw(&world, &camera);

                // Set immediate UI in redraw here
                gui.immediate_ui(|gui| {
                    let ctx = gui.context();
                    egui::CentralPanel::default().show(&ctx, |ui| {
    
                        let image_resp = ui.image(tex_id, Vec2::new(512.0,512.0));
    
                    });
                });
                // Render UI
                renderer.render(&mut gui);
            }
            Event::MainEventsCleared => {
                renderer.surface().window().request_redraw();
            }
            _ => (),
        }
    });
}
