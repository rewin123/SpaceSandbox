use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{CommandBuffer, DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};


use log::*;
use simplelog::*;
use tobj::LoadError;
use winit::window::Window;

use SpaceSandbox::*;
use SpaceSandbox::wavefront::load_gray_obj_now;

// for time measure wolfpld/tracy


fn main() {
    init_logger();

    let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop).unwrap();
    info!("Created window");

    let mut graphic_base = GraphicBase::init(window);

    let mut camera = RenderCamera::new(&graphic_base.allocator);
    camera.aspect = (graphic_base.swapchain.extent.width as f32) / (graphic_base.swapchain.extent.height as f32);
    camera.update_projectionmatrix();

    let mut gray_draw = GrayscalePipeline::new(&graphic_base, &camera).unwrap();

    info!("Tomokitty loading...");
    let mut scene = load_gray_obj_now(
        &graphic_base,
        String::from("res/test_res/models/tomokitty/sculpt.obj")).unwrap();

    let pools = Pools::init(
        &graphic_base.device,
        &graphic_base.queue_families
    ).unwrap();

    let command_buffers = create_commandbuffers(
        &graphic_base.device,
        &pools,
        graphic_base.swapchain.imageviews.len()
    ).unwrap();

    let mut gui = EguiWrapper::new(
        &graphic_base
    );

    use winit::event::{Event, WindowEvent};
    eventloop.run(move |event, _, controlflow| {

      gui.integration.handle_event(&event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                unsafe {
                    info!("Finishing.....");
                    graphic_base.device.device_wait_idle().expect("Waiting problem");
                }

                *controlflow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // doing the work here (later)
                graphic_base.window.request_redraw();
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

                    let frw = camera.view_direction;
                    let up = -camera.down_direction;
                    let right = camera.get_right_vector();
                    let dist = camera.position.magnitude();

                    let dp : nalgebra::Vector3<f32> = move_vector.x * right + up.scale( move_vector.y) + frw.scale(move_vector.z);
                    camera.position = camera.position + dp;
                    camera.position = camera.position.normalize().scale(dist);
                    camera.view_direction = -camera.position.normalize();
                    camera.down_direction = camera.view_direction.cross(&right);
                }
            }
            Event::RedrawRequested(_) => {
                //render here (later)
                let image_index = graphic_base.next_frame();

                unsafe {

                    gui.integration.begin_frame();

                    egui::Window::new("Loaded meshes")
                        .resizable(true)
                        .show(&gui.integration.context(), |ui| {

                            let mut del_mesh = None;
                            for (idx, m) in scene.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{} : {} verts", m.name.clone(), m.vertex_count));
                                    if ui.button("Delete").clicked() {
                                        del_mesh = Some(idx);
                                    }
                                });
                            }
                            match del_mesh {
                                Some(idx) => {
                                    scene.remove(idx);
                                }
                                None => {}
                            }


                        });
                    let (output, shapes) = gui.integration.end_frame(&mut graphic_base.window);
                    let clipped_meshes = gui.integration.context().tessellate(shapes);

                    camera.update_viewmatrix();
                    camera.update_inner_buffer();

                    unsafe {
                        graphic_base.device.begin_command_buffer(command_buffers[image_index as usize], &vk::CommandBufferBeginInfo::builder());
                    }

                    gray_draw.update_commandbuffer(
                        command_buffers[image_index as usize],
                        &graphic_base.device,
                        &graphic_base.swapchain,
                        &scene,
                        image_index as usize
                    );


                    gui.integration.paint(command_buffers[image_index as usize], image_index as usize, output, clipped_meshes);

                    unsafe {
                        graphic_base.device.end_command_buffer(command_buffers[image_index as usize]).unwrap();
                    }

                    graphic_base.end_frame(&command_buffers, image_index);
                };
            }
            _ => {}
        }
    });
}

