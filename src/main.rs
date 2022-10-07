use std::io::Read;
use std::sync::Arc;
use ash::{vk};
use ash::vk::{BufferUsageFlags};
use byteorder::ByteOrder;
use egui::RawInput;
use gltf::{Semantic};
use gltf::buffer::{Source};
use gltf::json::accessor::ComponentType;


use log::*;

use SpaceSandbox::*;
use SpaceSandbox::asset_server::{AssetServer, BaseModels};
use SpaceSandbox::MaterialTexture::{Diffuse, MetallicRoughness, Normal};
use SpaceSandbox::task_server::{TaskServer, TaskState};
use SpaceSandbox::ui::*;
use SpaceSandbox::game::*;

// for time measure wolfpld/tracy

fn init_rayon() {
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();
}

fn main() {
    init_logger();
    init_rayon();

    let mut game = Game::default();
    let mut assets = AssetServer::new(&game);

    let mut camera = RenderCamera::new(&game.gb.allocator);
    camera.aspect = (game.gb.swapchain.extent.width as f32) / (game.gb.swapchain.extent.height as f32);
    camera.update_projectionmatrix();

    let mut gray_draw = SingleTexturePipeline::new(&game.gb, &camera).unwrap();

    let mut gbuffer_draw = GBufferFillPipeline::new(&game.gb, &camera).unwrap();
    let mut light_draw = MeshLightPipeline::new(&game.gb, &camera).unwrap();
    let mut copy_pipe = TextureDemonstratePipeline::new(&game.gb.get_api_base(&game.pools));

    let mut fbs = vec![];
    for image in &game.gb.swapchain.images {

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(*image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::B8G8R8A8_UNORM)
            .subresource_range(*subresource_range);
        let imageview = unsafe {
            game.gb.device.create_image_view(&imageview_create_info, None).unwrap()
        };

        let iview = [imageview];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(light_draw.renderpass.inner)
            .attachments(&iview)
            .width(game.gb.swapchain.extent.width)
            .height(game.gb.swapchain.extent.height)
            .layers(1);
        let fb = unsafe { game.gb.device.create_framebuffer(&framebuffer_info, None) }.unwrap();
        fbs.push(Arc::new(FramebufferSafe {
            franebuffer: fb,
            images: vec![],
            renderpass: light_draw.renderpass.clone(),
            device: game.gb.device.clone()
        }));
    }

    game.render_server.point_lights.push(PointLight {
        intensity: 5.0,
        position: [0.0, 1.0, 0.0],
        color: [1.0, 1.0, 1.0],
        instance: BufferSafe::new(
            &game.gb.allocator,
            PointLight::get_instance_stride() as u64,
        BufferUsageFlags::VERTEX_BUFFER,
        gpu_allocator::MemoryLocation::CpuToGpu).unwrap()
    });

    let gbuffer = gbuffer_draw.create_framebuffer();
    // let light_buffer = light_draw.create_framebuffer();

    info!("Finish loading");

    unsafe {
        game.gb.device.device_wait_idle().unwrap();
    }

    let command_buffers = create_commandbuffers(
        &game.gb.device,
        &game.pools,
        game.gb.swapchain.imageviews.len()
    ).unwrap();

    let mut show_task_list = false;
    let mut show_gltf = true;
    let mut show_light_window = false;

    let mut fps_counter = FpsCounter::default();
    let mut api_window = ApiInfoWindow::new(&game.gb);
    let mut gltf_select = SelectGltfWindow::new(&assets);

    use winit::event::{Event, WindowEvent};

    for light in &mut game.render_server.point_lights {
        light.fill_instanse();
    }

    game.simple_loop(
     move |game, event, _, controlflow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                unsafe {
                    info!("Finishing.....");
                    game.gb.device.device_wait_idle().expect("Waiting problem");
                }

                *controlflow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // doing the work here (later)
                game.gb.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                    ..
                } = input
                {
                    let mut move_vector = nalgebra::Vector3::new(0.0, 0.0 ,0.0);
                    match keycode {
                        winit::event::VirtualKeyCode::Right => {
                            move_vector.x = 1.0;
                        }
                        winit::event::VirtualKeyCode::Left => {
                            move_vector.x = -1.0;
                        }
                        winit::event::VirtualKeyCode::Up => {
                            move_vector.y = 1.0;
                        }
                        winit::event::VirtualKeyCode::Down => {
                            move_vector.y = -1.0;
                        }
                        winit::event::VirtualKeyCode::PageUp => {
                            move_vector.z = 1.0;
                        }
                        winit::event::VirtualKeyCode::PageDown => {
                            move_vector.z = -1.0;
                        }
                        _ => {}
                    }

                    move_vector *= 0.1f32;
                    let frw = camera.view_direction;
                    let up = -camera.down_direction;
                    let right = camera.get_right_vector();
                    let mut dist = camera.position.magnitude();
                    dist += move_vector.z * dist;

                    let dp : nalgebra::Vector3<f32> = move_vector.x * right + up.scale( move_vector.y) + frw.scale(move_vector.z);
                    camera.position = camera.position + dp * dist;
                    camera.position = camera.position.normalize().scale(dist);
                    camera.view_direction = -camera.position.normalize();
                    camera.down_direction = camera.view_direction.cross(&right);

                    info!("{:#?}", camera.position);
                }
            }
            Event::RedrawRequested(_) => {
                //render here (later)
                // info!("Start frame!");
                let image_index = game.gb.next_frame();

                unsafe {

                    for light in &mut game.render_server.point_lights {
                        light.fill_instanse();
                    }

                    game.gui.integration.begin_frame();

                    egui::TopBottomPanel::top(0).show(&game.gui.integration.context(), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(format!("{} tasks running", game.task_server.get_task_count())).clicked() {
                                show_task_list = true;
                            }
                            if ui.button("Lights").clicked() {
                                show_light_window = true;
                            }
                            fps_counter.draw(ui);
                            api_window.draw(ui);
                        });
                    });

                    if show_light_window {
                        egui::Window::new("Lights").show(
                            &game.gui.integration.context(), |ui| {
                                for light in &mut game.render_server.point_lights {

                                    ui.add(egui::DragValue::new(&mut light.intensity));
                                    ui.add(egui::DragValue::new(&mut light.position[1]));
                                    ui.add(egui::DragValue::new(&mut light.position[0]));

                                    ui.separator();
                                }
                            }
                        );
                    }

                    if show_gltf {
                        egui::Window::new("Select gltf").show(
                            &game.gui.integration.context(), |ui| {
                                if gltf_select.draw(ui, &mut assets, game) {
                                    show_gltf = false;
                                }
                            }
                        );
                    }

                    if show_task_list {
                        let win_res = egui::Window::new("Task list")
                            .show(&game.gui.integration.context(), |ui| {

                            if ui.button("Close").clicked() {
                                show_task_list = false;
                            }
                            let tasks = game.task_server.clone_task_list();
                            for t in tasks {
                                let state = t.get_state();
                                match state {
                                    TaskState::Created => {
                                        ui.label(t.get_name());
                                    }
                                    TaskState::Running => {
                                        ui.colored_label(egui::color::Color32::GREEN, t.get_name());
                                    }
                                    TaskState::Finished => {
                                        ui.colored_label(egui::color::Color32::RED, t.get_name());
                                    }
                                }
                            }
                        });
                    }

                    let (gui_output, shapes) = game.gui.integration.end_frame(&mut game.gb.window);
                    let clipped_meshes = game.gui.integration.context().tessellate(shapes);

                    camera.update_viewmatrix();
                    camera.update_inner_buffer();

                    unsafe {
                        game.gb.device.begin_command_buffer(command_buffers[image_index as usize], &vk::CommandBufferBeginInfo::builder()).unwrap();
                    }


                    // gray_draw.update_commandbuffer(
                    //     command_buffers[image_index as usize],
                    //     &game.gb.device,
                    //     &game.gb.swapchain,
                    //     &game.render_server.render_models,
                    //     &assets.texture_server,
                    //     image_index as usize
                    // ).unwrap();

                    light_draw.set_camera(&camera);

                    gbuffer_draw.process(
                        command_buffers[image_index as usize],
                            &[],
                        &gbuffer,
                            &game.render_server,
                            &assets);

                    light_draw.process(
                        command_buffers[image_index as usize],
                        &gbuffer.images[0..4],
                        &fbs[image_index as usize],
                        &game.render_server,
                        &assets
                    );

                    game.gui.integration.paint(
                        command_buffers[image_index as usize],
                        image_index as usize,
                        gui_output,
                        clipped_meshes);
                    unsafe {
                        game.gb.device.end_command_buffer(command_buffers[image_index as usize]).unwrap();
                    }
                    game.gb.end_frame(&command_buffers, image_index);

                    assets.texture_server.sync_tick();

                    unsafe {
                        // info!("Wait device");
                        // game.gb.device.device_wait_idle().unwrap();
                    }
                };
            }
            _ => {}
        }
    });
}

