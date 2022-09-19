use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{CommandBuffer, DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, RenderPass, SurfaceKHR, SwapchainKHR};


use log::*;
use simplelog::*;
// use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

pub mod safe_warp;
pub mod debug_layer;
pub mod vulkan_init_utils;
pub mod example_pipeline;
pub mod gui;
pub mod camera;
pub mod grayscale_pipeline;

pub use debug_layer::*;
pub use vulkan_init_utils::*;
use example_pipeline::*;
pub use gui::*;
pub use camera::*;
pub use safe_warp::*;
pub use grayscale_pipeline::*;

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

pub struct AllocatorSafe {
    pub inner : vk_mem::Allocator
}

impl Deref for AllocatorSafe {
    type Target = vk_mem::Allocator;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for AllocatorSafe {
    fn drop(&mut self) {
        info!("Destroy allocator");
        unsafe {
            self.inner.destroy();
        }
    }
}

impl Drop for GraphicBase {
    fn drop(&mut self) {
        info!("Destroy GraphicBase");
    }
}

pub struct QueueFamilies {
    graphics_q_index: u32,
    transfer_q_index: u32,
}

pub struct Queues {
    pub graphics_queue: vk::Queue,
    pub transfer_queue: vk::Queue,
}

pub struct DeviceSafe {
    pub inner : Device,
    instance : Arc<InstanceSafe>
}

impl Drop for DeviceSafe {
    fn drop(&mut self) {
        info!("Destroy device");
        unsafe {
            self.inner.destroy_device(None);
        }
    }
}

impl Deref for DeviceSafe {
    type Target = Device;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct RenderPassSafe {
    pub inner : RenderPass,
    device : Arc<DeviceSafe>
}

impl Drop for RenderPassSafe {
    fn drop(&mut self) {
        info!("Destroy RenderPass");
        unsafe {
            self.device.destroy_render_pass(self.inner, None);
        }
    }
}

impl Deref for RenderPassSafe {
    type Target = RenderPass;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}




pub fn init_renderpass(
   base : &GraphicBase
) -> Result<RenderPassSafe, vk::Result> {
    let attachments = [vk::AttachmentDescription::builder()
        .format(
            base.surfaces
                .get_formats(base.physical_device)?
                .first()
                .unwrap()
                .format,
        )
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .samples(vk::SampleCountFlags::TYPE_1)
        .build(),
        vk::AttachmentDescription::builder()
            .format(vk::Format::D32_SFLOAT)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1)
            .build(),];
    let color_attachment_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let depth_attachment_reference = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    };
    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_references)
        .depth_stencil_attachment(&depth_attachment_reference)
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .build()];
    let subpass_dependencies = [vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_subpass(0)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(
            vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        )
        .build()];
    let renderpass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&subpass_dependencies);
    let renderpass = unsafe { base.device.create_render_pass(&renderpass_info, None)? };

    Ok(base.wrap_render_pass(renderpass))
}

pub struct Pools {
    pub commandpool_graphics: vk::CommandPool,
    pub commandpool_transfer: vk::CommandPool,
    device : Arc<DeviceSafe>
}

impl Drop for Pools {
    fn drop(&mut self) {
        info!("Destroy command pools");
        unsafe {
            self.device.destroy_command_pool(self.commandpool_graphics, None);
            self.device.destroy_command_pool(self.commandpool_transfer, None);
        }
    }
}

impl Pools {
    pub fn init(
        logical_device: &Arc<DeviceSafe>,
        queue_families: &QueueFamilies,
    ) -> Result<Pools, vk::Result> {
        let graphics_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.graphics_q_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_graphics =
            unsafe { logical_device.create_command_pool(&graphics_commandpool_info, None) }?;
        let transfer_commandpool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_families.transfer_q_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let commandpool_transfer =
            unsafe { logical_device.create_command_pool(&transfer_commandpool_info, None) }?;

        Ok(Pools {
            commandpool_graphics,
            commandpool_transfer,
            device : logical_device.clone()
        })
    }
}

pub fn create_commandbuffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.commandpool_graphics)
        .command_buffer_count(amount as u32);
    unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}


use nalgebra as na;
use tobj::LoadError;
use vk_mem::ffi::VkResult;
use winit::window::Window;
use crate::safe_warp::InstanceSafe;


pub struct GPUMesh {
    pub pos_data : BufferSafe,
    pub normal_data : BufferSafe,
    pub index_data : BufferSafe,
    pub vertex_count : u32
}


pub fn init_logger() {
    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("detailed.log").unwrap())
        ]
    );
}

pub fn load_gray_obj_now(graphic_base : &GraphicBase, path : String) -> Result<Vec<GPUMesh>, LoadError> {
    let (models, materials) = tobj::load_obj(path,
                                             &tobj::GPU_LOAD_OPTIONS)?;

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

    Ok(scene)
}