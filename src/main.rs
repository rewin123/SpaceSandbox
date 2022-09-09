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

    let mut graphic_base = GraphicBase::init(window);

    let mut renderpass = init_renderpass(&graphic_base).unwrap();

    graphic_base.swapchain.create_framebuffers(
        &graphic_base.device,
                    renderpass.inner);

    unsafe {

    };
}
