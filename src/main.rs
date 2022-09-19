use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};


use log::*;
use simplelog::*;
use winit::window::Window;

use SpaceSandbox::*;
use SpaceSandbox::example_pipeline::ExamplePipeline;

// for time measure wolfpld/tracy


fn main() {
    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("detailed.log").unwrap())
        ]
    );



    let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop).unwrap();
    info!("Created window");

    let mut graphic_base = GraphicBase::init(window);

    let mut renderpass = init_renderpass(&graphic_base).unwrap();

    graphic_base.swapchain.create_framebuffers(
        &graphic_base.device,
                    renderpass.inner);

    info!("Tomokitty loading...");
    let (models, materials) = tobj::load_obj("res/test_res/models/tomokitty/sculpt.obj",
                                             &tobj::GPU_LOAD_OPTIONS).expect("Problem with loading model");

    let mut scene = vec![];


    for (i, m) in models.iter().enumerate() {
        info!("Found model {}!", m.name.clone());

        let mesh = &m.mesh;

        let mut chandeg_pos = vec![];
        for vertex_idx in 0..(mesh.positions.len() / 3) {
            chandeg_pos.push(mesh.positions[vertex_idx * 3]);
            chandeg_pos.push(mesh.positions[vertex_idx * 3 + 1]);
            chandeg_pos.push(mesh.positions[vertex_idx * 3 + 2]);
            chandeg_pos.push(1.0);
        }


        let mut pos_data = BufferSafe::new(
            &graphic_base.allocator,
            (chandeg_pos.len() * 4) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu
        ).unwrap();

        let mut index_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.indices.len() * 4) as u64,
            vk::BufferUsageFlags::INDEX_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu
        ).unwrap();

        let mut normal_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.normals.len() * 3) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            vk_mem::MemoryUsage::CpuToGpu
        ).unwrap();

        pos_data.fill(&chandeg_pos).unwrap();
        index_data.fill(&mesh.indices).unwrap();
        normal_data.fill(&mesh.normals).unwrap();

        scene.push(
            GPUMesh {
                pos_data,
                index_data,
                normal_data,
                vertex_count: mesh.indices.len() as u32,
            }
        );
    }

    let mut uniformbuffer = BufferSafe::new(
        &graphic_base.allocator,
        64 * 2,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk_mem::MemoryUsage::CpuToGpu
    ).unwrap();
    let cameratransform: [[[f32; 4]; 4]; 2] = [
        nalgebra::Matrix4::identity().into(),
        nalgebra::Matrix4::identity().into()
    ];
    uniformbuffer.fill(&cameratransform).unwrap();


    let pipeline = ExamplePipeline::init(
        &graphic_base.device,
        &graphic_base.swapchain,
        &renderpass).unwrap();


    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty : vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count : graphic_base.swapchain.amount_of_images
        }
    ];
    let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
        .max_sets(graphic_base.swapchain.amount_of_images)
        .pool_sizes(&pool_sizes);
    let descriptor_pool = unsafe {
        graphic_base.device.create_descriptor_pool(&descriptor_pool_info, None)
    }.unwrap();

    let desc_layouts =
        vec![pipeline.descriptor_set_layouts[0]; graphic_base.swapchain.amount_of_images as usize];
    let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&desc_layouts);
    let descriptor_sets =
        unsafe { graphic_base.device.allocate_descriptor_sets(&descriptor_set_allocate_info)
        }.unwrap();

    let mut camera = Camera::default();
    camera.aspect = (graphic_base.swapchain.extent.width as f32) / (graphic_base.swapchain.extent.height as f32);
    camera.update_projectionmatrix();

    for (i, descset) in descriptor_sets.iter().enumerate() {
        let buffer_infos = [vk::DescriptorBufferInfo {
            buffer: uniformbuffer.buffer,
            offset: 0,
            range: 128,
        }];
        let desc_sets_write = [vk::WriteDescriptorSet::builder()
            .dst_set(*descset)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&buffer_infos)
            .build()];
        unsafe { graphic_base.device.update_descriptor_sets(&desc_sets_write, &[]) };
    }

    let pools = Pools::init(
        &graphic_base.device,
        &graphic_base.queue_families
    ).unwrap();

    let command_buffers = create_commandbuffers(
        &graphic_base.device,
        &pools,
        graphic_base.swapchain.framebuffers.len()
    ).unwrap();

    fill_commandbuffers(
        &command_buffers,
        &graphic_base.device,
        &renderpass,
        &graphic_base.swapchain,
        &pipeline,
        &scene,
        &descriptor_sets
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
                    graphic_base.
                        device
                        .wait_for_fences(
                            &[graphic_base.swapchain.may_begin_drawing[graphic_base.swapchain.current_image]],
                            true,
                            std::u64::MAX
                        )
                        .expect("fence waiting problem");

                    graphic_base
                        .device
                        .reset_fences(
                            &[graphic_base.swapchain.may_begin_drawing[graphic_base.swapchain.current_image]])
                        .expect("rest fences");

                    camera.update_viewmatrix();
                    camera.update_buffer(&mut uniformbuffer);

                    unsafe {
                        graphic_base.device.begin_command_buffer(command_buffers[image_index as usize], &vk::CommandBufferBeginInfo::builder());
                    }
                    update_commandbuffer(
                        command_buffers[image_index as usize],
                        &graphic_base.device,
                        &renderpass,
                        &graphic_base.swapchain,
                        &pipeline,
                        &scene,
                        &descriptor_sets,
                        image_index as usize
                    );

                    let mut style = egui::Style::default();
                    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
                    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke { width: 1.0, color: egui::Color32::BLACK };
                    style.visuals.widgets.active.bg_fill = egui::Color32::WHITE;
                    style.visuals.widgets.active.fg_stroke = egui::Stroke { width: 1.0, color: egui::Color32::BLACK };
                    style.visuals.widgets.inactive.bg_fill = egui::Color32::LIGHT_BLUE;
                    style.visuals.widgets.inactive.fg_stroke = egui::Stroke { width: 1.0, color: egui::Color32::BLACK };
                    gui.integration.context().set_style(style);

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

                    let semaphores_available = [graphic_base.swapchain.image_available[graphic_base.swapchain.current_image]];
                    let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                    let semaphores_finished = [graphic_base.swapchain.rendering_finished[graphic_base.swapchain.current_image]];
                    let commandbuffers = [command_buffers[image_index as usize]];
                    let submit_info = [vk::SubmitInfo::builder()
                        .wait_semaphores(&semaphores_available)
                        .wait_dst_stage_mask(&waiting_stages)
                        .command_buffers(&commandbuffers)
                        .signal_semaphores(&semaphores_finished)
                        .build()];

                    unsafe {
                        graphic_base
                            .device
                            .queue_submit(
                                graphic_base.queues.graphics_queue,
                                &submit_info,
                                graphic_base.swapchain.may_begin_drawing[graphic_base.swapchain.current_image],
                            )
                            .expect("queue submission");
                    };


                    let swapchains = [graphic_base.swapchain.inner];
                    let indices = [image_index];
                    let present_info = vk::PresentInfoKHR::builder()
                        .wait_semaphores(&semaphores_finished)
                        .swapchains(&swapchains)
                        .image_indices(&indices);
                    unsafe {
                        graphic_base
                            .swapchain
                            .loader
                            .queue_present(graphic_base.queues.graphics_queue, &present_info)
                            .expect("queue presentation");
                    };
                };
            }
            _ => {}
        }
    });
}
