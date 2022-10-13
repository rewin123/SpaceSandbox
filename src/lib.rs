use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use ash::{Device, Entry, vk};
use ash::extensions::{khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{PhysicalDevice, PhysicalDeviceProperties, RenderPass, SurfaceKHR, SwapchainKHR, DescriptorPool, VertexInputBindingDescription};

use log::*;
use simplelog::*;
use std::default::Default;
use gpu_allocator::vulkan::{Allocation, AllocatorCreateDesc};
// use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

pub mod safe_warp;
pub mod debug_layer;
pub mod vulkan_init_utils;
pub mod gui;
pub mod camera;
pub mod graphic_base;
pub mod assets;
pub mod pipelines;
pub mod task_server;
pub mod ui;
pub mod game;
pub mod light;

pub use debug_layer::*;
pub use vulkan_init_utils::*;
pub use gui::*;
pub use camera::*;
pub use safe_warp::*;
pub use graphic_base::*;
pub use assets::runtime_gpu_assets::*;
pub use assets::*;
pub use pipelines::*;
pub use game::*;


pub struct AllocatorSafe {
    pub device : Arc<DeviceSafe>,
    pub inner : Mutex<gpu_allocator::vulkan::Allocator>
}

impl AllocatorSafe {
    pub fn allocate(&self, desc : &gpu_allocator::vulkan::AllocationCreateDesc)
            -> gpu_allocator::Result<Allocation> {
        let mut lock = self.inner.lock().unwrap();
        lock.allocate(desc)
    }

    pub fn free(&self, allocation : gpu_allocator::vulkan::Allocation) {
        let mut lock = self.inner.lock().unwrap();
        lock.free(allocation);
    }
}


impl Drop for AllocatorSafe {
    fn drop(&mut self) {
        info!("Destroy allocator");
    }
}

impl Drop for GraphicBase {
    fn drop(&mut self) {
        info!("Destroy GraphicBase");
    }
}

#[derive(Clone)]
pub struct QueueFamilies {
    graphics_q_index: u32,
    transfer_q_index: u32,
}

#[derive(Clone)]
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

pub struct Pool {
    pub pool : vk::CommandPool,
    device : Arc<DeviceSafe>
}

impl Deref for Pool {
    type Target = vk::CommandPool;

    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl Drop for Pool {
    fn drop(&mut self) {
        unsafe {
            debug!("Destroy pool");
            self.device.destroy_command_pool(self.pool, None);
        }
    }
}

#[derive(Clone)]
pub struct Pools {
    pub graphics: Arc<Pool>,
    pub transfer: Arc<Pool>
}

impl Drop for Pools {
    fn drop(&mut self) {
        
    }
}

impl Pools {
    pub fn init(
        logical_device: &Arc<DeviceSafe>,
        queue_families: &QueueFamilies,
    ) -> Result<Arc<Pools>, vk::Result> {
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

        Ok(Arc::new(Pools {
            graphics : Arc::new(Pool {
                pool : commandpool_graphics,
                device : logical_device.clone()
            }),
            transfer : Arc::new(
                Pool {
                    pool : commandpool_transfer,
                    device : logical_device.clone()
                }
            )
        }))
    }
}

pub fn create_commandbuffers(
    logical_device: &ash::Device,
    pools: &Pools,
    amount: usize,
) -> Result<Vec<vk::CommandBuffer>, vk::Result> {
    let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pools.graphics.pool)
        .command_buffer_count(amount as u32);
    unsafe { logical_device.allocate_command_buffers(&commandbuf_allocate_info) }
}

use winit::window::Window;
use crate::safe_warp::InstanceSafe;


pub struct GPUMesh {
    pub pos_data : BufferSafe,
    pub normal_data : BufferSafe,
    pub tangent_data: BufferSafe,
    pub uv_data : BufferSafe,
    pub index_data : BufferSafe,
    pub vertex_count : u32,
    pub name : String
}

impl GPUMesh {
    pub fn get_vertex_attrib_desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 1,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 2,
                location: 2,
                offset: 0,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 3,
                location: 3,
                offset: 0,
                format: vk::Format::R32G32_SFLOAT
            },

            //define instance buffer
            vk::VertexInputAttributeDescription {
                binding: 4,
                location: 4,
                offset: 0,
                format: vk::Format::R32G32B32A32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 4,
                location: 5,
                offset: 16,
                format: vk::Format::R32G32B32A32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 4,
                location: 6,
                offset: 32,
                format: vk::Format::R32G32B32A32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding: 4,
                location: 7,
                offset: 48,
                format: vk::Format::R32G32B32A32_SFLOAT
            }
        ]
    }

    pub fn get_binding_desc() -> Vec<VertexInputBindingDescription> {
        vec![
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: 4 * 3,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            vk::VertexInputBindingDescription {
                binding: 1,
                stride: 4 * 3,
                input_rate: vk::VertexInputRate::VERTEX
            },
            vk::VertexInputBindingDescription {
                binding: 2,
                stride: 4 * 3,
                input_rate: vk::VertexInputRate::VERTEX
            },
            vk::VertexInputBindingDescription {
                binding: 3,
                stride: 4 * 2,
                input_rate: vk::VertexInputRate::VERTEX
            },
            vk::VertexInputBindingDescription {
                binding: 4,
                stride: 4 * 16,
                input_rate: vk::VertexInputRate::INSTANCE
            }
        ]
    }
}

pub struct Material {
    pub color : ServerTexture,
    pub normal : ServerTexture,
    pub metallic_roughness: ServerTexture
}

pub struct RenderModel {
    pub mesh : Arc<GPUMesh>,
    pub instances : BufferSafe,
    pub material : Material,
    pub cache_size : usize,
    pub model_count : usize,
    pub cpu_cache : Vec<f32>
}

pub struct MeshLight {
    pub mesh : Arc<GPUMesh>,
}

impl RenderModel {
    pub fn new(
        allocator : &Arc<AllocatorSafe>,
        mesh : Arc<GPUMesh>,
        material : Material
    ) -> Self {
        let cache_size = 10;

        let instances = BufferSafe::new(
            allocator,
            (cache_size * 4 * 16) as u64,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu
        ).unwrap();

        Self {
            mesh,
            material,
            cache_size,
            model_count : 0,
            instances,
            cpu_cache : vec![]
        }
    }

    pub fn update_instance_buffer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.instances.fill(&self.cpu_cache)?;

        Ok(())
    }

    pub fn add_matrix(&mut self, matrix : &[[f32; 4]; 4]) {
        for y in 0..4 {
            for x in 0..4 {
                self.cpu_cache.push(matrix[y][x]);
            }
        }
        self.model_count += 1;
    }

    pub fn clear_cache(&mut self) {
        self.cpu_cache.clear();
        self.model_count = 0;
    }
}

pub fn init_logger() {
    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Debug, Config::default(), File::create("detailed.log").unwrap())
        ]
    );
}



pub struct DescriptorPoolSafe {
    pub pool : DescriptorPool,
    pub device : Arc<DeviceSafe>
}

impl Deref for DescriptorPoolSafe {
    type Target = DescriptorPool;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl Drop for DescriptorPoolSafe {
    fn drop(&mut self) {
        unsafe {
            info!("Destroy descriptor pool");
            self.device.destroy_descriptor_pool(self.pool, None);
        }
    }
}
