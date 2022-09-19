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
use winit::platform::unix::WindowExtUnix;
use winit::window::Window;

use SpaceSandbox::*;
use SpaceSandbox::example_pipeline::ExamplePipeline;

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
    let scene = load_gray_obj_now(
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
                    match keycode {
                        winit::event::VirtualKeyCode::Right => {
                            camera.turn_right(0.1);
                        }
                        winit::event::VirtualKeyCode::Left => {
                            camera.turn_left(0.1);
                        }
                        winit::event::VirtualKeyCode::Up => {
                            camera.move_forward(0.05);
                        }
                        winit::event::VirtualKeyCode::Down => {
                            camera.move_backward(0.05);
                        }
                        winit::event::VirtualKeyCode::PageUp => {
                            camera.turn_up(0.02);
                        }
                        winit::event::VirtualKeyCode::PageDown => {
                            camera.turn_down(0.02);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                //render here (later)
                graphic_base.swapchain.current_image =
                    (graphic_base.swapchain.current_image + 1) % graphic_base.swapchain.amount_of_images as usize;

                let (image_index, _) = unsafe {
                    graphic_base
                        .swapchain
                        .loader
                        .acquire_next_image(
                            graphic_base.swapchain.inner,
                            std::u64::MAX,
                            graphic_base.swapchain.image_available[graphic_base.swapchain.current_image],
                            vk::Fence::null()
                        )
                        .expect("image acquisition trouble")
                };

                unsafe {
                    graphic_base.start_frame();

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

                    gui.integration.begin_frame();

                    egui::Window::new("Test window")
                            .resizable(true)
                            .show(&gui.integration.context(), |ui| {
                        ui.label("Hello world");
                        ui.button("Its a button");


                    });
                    let (_, shapes) = gui.integration.end_frame(&mut graphic_base.window);
                    let clipped_meshes = gui.integration.context().tessellate(shapes);
                    gui.integration.paint(command_buffers[image_index as usize], image_index as usize, clipped_meshes);

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

