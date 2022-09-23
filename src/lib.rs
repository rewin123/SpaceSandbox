use std::fs::File;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{BufferUsageFlags, CommandBuffer, DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, RenderPass, SurfaceKHR, SwapchainKHR};



use log::*;
use simplelog::*;
use std::default::Default;
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
pub mod graphic_base;
pub mod assets;
pub mod render_server;
pub mod single_texture_pipeline;

pub use debug_layer::*;
pub use vulkan_init_utils::*;
use example_pipeline::*;
pub use gui::*;
pub use camera::*;
pub use safe_warp::*;
pub use grayscale_pipeline::*;
pub use graphic_base::*;
pub use assets::runtime_gpu_assets::*;
pub use assets::*;
pub use single_texture_pipeline::*;


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
use nalgebra_glm::all;
use tobj::LoadError;
use vk_mem::ffi::VkResult;
use vk_mem::MemoryUsage;
use winit::window::Window;
use crate::safe_warp::InstanceSafe;


pub struct GPUMesh {
    pub pos_data : BufferSafe,
    pub normal_data : BufferSafe,
    pub uv_data : BufferSafe,
    pub index_data : BufferSafe,
    pub vertex_count : u32,
    pub name : String
}

pub struct Material {
    pub color : Arc<TextureSafe>
}

pub struct RenderModel {
    pub mesh : GPUMesh,
    pub material : Material
}

pub struct TextureSafe {
    image : vk::Image,
    allocation : vk_mem::Allocation,
    allocation_info : vk_mem::AllocationInfo,
    imageview : vk::ImageView,
    sampler : vk::Sampler,
    allocator : Arc<AllocatorSafe>,
    device : Arc<DeviceSafe>
}

impl Drop for TextureSafe {
    fn drop(&mut self) {

        unsafe {
            debug!("Destroy TextureSafe");
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.imageview, None);
            self.allocator.destroy_image(self.image, &self.allocation).unwrap();
        }
    }
}

impl TextureSafe {

    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
        gb: &GraphicBase,
        pools : &Pools) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)
            .map(|img| img.to_rgba())
            .expect("unable to open image");
        let (width, height) = image.dimensions();
        let mut res = TextureSafe::new(
            &gb.allocator,
            &gb.device,
            vk::Extent2D {
                width,
                height
            },
            vk::Format::R8G8B8A8_SRGB
        );

        let data = image.clone().into_raw();
        info!("data len {}", data.len());
        let mut buffer = ManuallyDrop::new( BufferSafe::new(
            &gb.allocator,
            data.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuToGpu,
        ).unwrap());
        buffer.fill(&data).unwrap();

        let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pools.commandpool_graphics)
            .command_buffer_count(1);
        let copycmdbuffer = unsafe {
            gb
                .device
                .allocate_command_buffers(&commandbuf_allocate_info)
        }
            .unwrap()[0];

        let cmdbegininfo = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            gb
                .device
                .begin_command_buffer(copycmdbuffer, &cmdbegininfo)
        }?;


        let barrier = vk::ImageMemoryBarrier::builder()
            .image(res.image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
        unsafe {
            gb.device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };


        //Insert commands here.
        let image_subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        };
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            image_subresource,
            ..Default::default()
        };
        unsafe {
            gb.device.cmd_copy_buffer_to_image(
                copycmdbuffer,
                buffer.buffer,
                res.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        let barrier = vk::ImageMemoryBarrier::builder()
            .image(res.image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
        unsafe {
            gb.device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };


        unsafe { gb.device.end_command_buffer(copycmdbuffer) }?;
        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&[copycmdbuffer])
            .build()];
        let fence = unsafe {
            gb
                .device
                .create_fence(&vk::FenceCreateInfo::default(), None)
        }?;
        unsafe {
            gb
                .device
                .queue_submit(gb.queues.graphics_queue, &submit_infos, fence)
        }?;
        unsafe { gb.device.wait_for_fences(&[fence], true, std::u64::MAX) }?;


        unsafe { gb.device.destroy_fence(fence, None) };
        // gb.allocator.destroy_buffer(buffer.buffer, &buffer.allocation)?;
        unsafe {
            gb
                .device
                .free_command_buffers(pools.commandpool_graphics, &[copycmdbuffer])
        };

        unsafe {
            gb.device.device_wait_idle().unwrap();
            }
    

        info!("Finish copy");

        unsafe {
            ManuallyDrop::drop(&mut buffer);
        }

        Ok(res)
    }

    fn new(
        allocator : &Arc<AllocatorSafe>,
        device : &Arc<DeviceSafe>,
        extent : vk::Extent2D,
        format : vk::Format) -> Self {
        let img_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width : extent.width,
                height : extent.height,
                depth : 1
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED);
        let alloc_create_info = vk_mem::AllocationCreateInfo {
            usage : vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (vk_image, allocation, allocation_info) = allocator
            .create_image(&img_create_info, &alloc_create_info)
            .expect("creating vkImage for texture");
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(vk_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });
        let imageview = unsafe {
            device.create_image_view(&view_create_info, None).expect("image view creaton")
        };
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR);
        let sampler =
            unsafe { device.create_sampler(&sampler_info, None) }.expect("sampler creation");
        Self {
            image : vk_image,
            allocation,
            allocation_info,
            imageview,
            sampler,
            allocator : allocator.clone(),
            device : device.clone()
        }
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

        let mut uv_data = BufferSafe::new(
            &graphic_base.allocator,
            (mesh.normals.len() * 4) as u64,
            BufferUsageFlags::VERTEX_BUFFER,
            MemoryUsage::CpuToGpu
        ).unwrap();


        pos_data.fill(&chandeg_pos).unwrap();
        index_data.fill(&mesh.indices).unwrap();
        normal_data.fill(&mesh.normals).unwrap();
        uv_data.fill(&vec![0.0f32; mesh.normals.len()]).unwrap();

        scene.push(
            GPUMesh {
                pos_data,
                index_data,
                normal_data,
                uv_data,
                vertex_count: mesh.indices.len() as u32,
                name : m.name.clone()
            }
        );
    }

    Ok(scene)
}