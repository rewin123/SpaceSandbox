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

    let graphic_base = GraphicBase::init(window);

    let swapchain_images = unsafe {
        graphic_base.swapchain.loader.get_swapchain_images(graphic_base.swapchain.inner).unwrap()
    };
    let mut swapchain_imageviews = Vec::with_capacity(swapchain_images.len());
    for image in &swapchain_images {
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
            graphic_base.device.create_image_view(&imageview_create_info, None).unwrap()
        };
        swapchain_imageviews.push(imageview);
    }

    unsafe {
        for iv in &swapchain_imageviews {
            graphic_base.device.destroy_image_view(*iv, None);
        }
        graphic_base.device.destroy_device(None);
    };
}
