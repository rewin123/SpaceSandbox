use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, RenderPass, SurfaceKHR, SwapchainKHR};

use log::*;
use simplelog::*;
use winit::platform::unix::WindowExtUnix;
use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

pub mod swapchain_safe;
pub mod surface_safe;
pub mod instance_safe;
pub mod debug_layer;
pub mod vulkan_init_utils;

pub use swapchain_safe::*;
pub use surface_safe::*;
pub use instance_safe::*;
pub use debug_layer::*;
pub use vulkan_init_utils::*;

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
        let device = Arc::new(DeviceSafe {inner : logical_device, instance : instance.clone()});

        let surface = Arc::new(SurfaceSafe::new(&window, &instance, &entry));

        let swapchain = SwapchainSafe::new(
            &surface,
            physical_device,
            &qfamindices,
            &device,
            &instance);

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
            swapchain
        }
    }

    pub fn wrap_render_pass(&self, pass : RenderPass) -> RenderPassSafe {
        RenderPassSafe {
            inner : pass,
            device : self.device.clone()
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
    graphics_queue: vk::Queue,
    transfer_queue: vk::Queue,
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
        .build()];
    let color_attachment_references = [vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    }];
    let subpasses = [vk::SubpassDescription::builder()
        .color_attachments(&color_attachment_references)
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





