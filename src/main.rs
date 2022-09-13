use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};

use log::*;
use simplelog::*;
use winit::platform::unix::WindowExtUnix;
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

    let allocation_create_info = vk_mem::AllocationCreateInfo {
        usage: vk_mem::MemoryUsage::CpuToGpu,
        ..Default::default()
    };
    let (buffer, allocation, allocation_info) = graphic_base.allocator.create_buffer(
        &ash::vk::BufferCreateInfo::builder()
            .size(16)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .build(),
        &allocation_create_info,
    ).unwrap();

    let data_ptr = graphic_base.allocator.map_memory(&allocation).unwrap() as *mut f32;
    let data = [-0.5f32, 0.0f32, 0.0f32, 1.0f32];
    unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), 4) };
    graphic_base.allocator.unmap_memory(&allocation);

    let pipeline = ExamplePipeline::init(
        &graphic_base.device,
        &graphic_base.swapchain,
        &renderpass).unwrap();

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
        &buffer
    ).unwrap();


    use winit::event::{Event, WindowEvent};
    eventloop.run(move |event, _, controlflow| match event {
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
    });
}
