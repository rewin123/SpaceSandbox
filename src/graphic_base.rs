use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Entry, vk};
use ash::vk::{CommandBuffer, PhysicalDevice, RenderPass};
use log::info;
use winit::window::Window;
use crate::{AllocatorSafe, DebugDongXi, DeviceSafe, GetDefaultPhysicalDevice, GetGraphicQueue, GetLogicalDevice, init_instance, InstanceSafe, QueueFamilies, Queues, RenderPassSafe, SurfaceSafe, SwapchainSafe, Pools};

#[derive(Clone)]
pub struct ApiBase {
    pub device : Arc<DeviceSafe>,
    pub allocator : Arc<AllocatorSafe>,
    pub queues : Queues,
    pub queue_families : QueueFamilies,
    pub pools : Pools
}

pub struct GraphicBase {
    pub instance : Arc<InstanceSafe>,
    pub debug : DebugDongXi,
    pub surfaces : Arc<SurfaceSafe>,
    pub physical_device : PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub queue_families : QueueFamilies,
    pub queues : Queues,
    pub device : Arc<DeviceSafe>,
    pub swapchain : SwapchainSafe,

    pub window : winit::window::Window,
    pub entry : Entry,
    pub allocator : Arc<AllocatorSafe>,
}

impl GraphicBase {
    pub fn init(window : Window) -> Self {
        let entry = unsafe {ash::Entry::load().unwrap() };

        let mut extension_name_pointers : Vec<*const c_char> =
            ash_window::enumerate_required_extensions(&window).unwrap()
                .iter()
                .map(|&name| name.as_ptr())
                .collect();


        let layer_names = vec!["VK_LAYER_KHRONOS_validation"];
        let instance = Arc::new(init_instance(&entry, &layer_names, &window));
        let debug = DebugDongXi::init(&entry, &instance).unwrap();

        let (physical_device, physical_device_properties) = GetDefaultPhysicalDevice(&instance);

        let qfamindices = GetGraphicQueue(&instance, &physical_device);
        let (logical_device, queues) = GetLogicalDevice(
            &layer_names,
            &instance,
            physical_device,
            &qfamindices);
        let device = Arc::new(DeviceSafe {inner : logical_device.clone(), instance : instance.clone()});

        let surface = Arc::new(SurfaceSafe::new(&window, &instance, &entry));


        info!("Creating allocator create info...");

        let allocator_create_info = vk_mem::AllocatorCreateInfo {
            physical_device,
            device: logical_device.clone(),
            instance: instance.inner.clone(),
            flags: Default::default(),
            preferred_large_heap_block_size: 0,
            frame_in_use_count: 0,
            heap_size_limits: None
        };
        info!("Creating allocator...");
        let mut allocator =
            Arc::new(AllocatorSafe {
                inner : vk_mem::Allocator::new(&allocator_create_info).unwrap()
            });

        let swapchain = SwapchainSafe::new(
            &surface,
            physical_device,
            &qfamindices,
            &device,
            &instance,
            &allocator);




        info!("Finished creating GraphicBase");

        Self {
            window,
            entry,
            instance,
            debug,
            surfaces : surface,
            physical_device,
            physical_device_properties,
            queue_families : qfamindices,
            queues,
            device,
            swapchain,
            allocator
        }
    }

    pub fn get_api_base(&self, pools : &Pools) -> ApiBase {
        ApiBase { 
            device: self.device.clone(), 
            allocator: self.allocator.clone(), 
            queues: self.queues.clone(),
            pools : pools.clone(),
            queue_families : self.queue_families.clone()
        }
    }

    pub fn wrap_render_pass(&self, pass : RenderPass) -> RenderPassSafe {
        RenderPassSafe {
            inner : pass,
            device : self.device.clone()
        }
    }

    pub fn start_frame(&self) {
        unsafe {
            self.
                device
                .wait_for_fences(
                    &[self.swapchain.may_begin_drawing[self.swapchain.current_image]],
                    true,
                    std::u64::MAX
                )
                .expect("fence waiting problem");

            self
                .device
                .reset_fences(
                    &[self.swapchain.may_begin_drawing[self.swapchain.current_image]])
                .expect("rest fences");
        }
    }


    pub fn end_frame(&self, command_buffers: &Vec<CommandBuffer>, image_index: u32) {
        let semaphores_available = [self.swapchain.image_available[self.swapchain.current_image]];
        let waiting_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let semaphores_finished = [self.swapchain.rendering_finished[self.swapchain.current_image]];
        let commandbuffers = [command_buffers[image_index as usize]];
        let submit_info = [vk::SubmitInfo::builder()
            .wait_semaphores(&semaphores_available)
            .wait_dst_stage_mask(&waiting_stages)
            .command_buffers(&commandbuffers)
            .signal_semaphores(&semaphores_finished)
            .build()];

        unsafe {
            self
                .device
                .queue_submit(
                    self.queues.graphics_queue,
                    &submit_info,
                    self.swapchain.may_begin_drawing[self.swapchain.current_image],
                )
                .expect("queue submission");
        };


        let swapchains = [self.swapchain.inner];
        let indices = [image_index];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&semaphores_finished)
            .swapchains(&swapchains)
            .image_indices(&indices);
        unsafe {
            self
                .swapchain
                .loader
                .queue_present(self.queues.graphics_queue, &present_info)
                .expect("queue presentation");
        };
    }

    pub fn next_frame(&mut self) -> u32 {
        self.swapchain.current_image =
            (self.swapchain.current_image + 1) % self.swapchain.amount_of_images as usize;

        let (image_index, _) = unsafe {
            self
                .swapchain
                .loader
                .acquire_next_image(
                    self.swapchain.inner,
                    std::u64::MAX,
                    self.swapchain.image_available[self.swapchain.current_image],
                    vk::Fence::null()
                )
                .expect("image acquisition trouble")
        };

        self.start_frame();

        return image_index;
    }
}
