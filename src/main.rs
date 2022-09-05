use std::fs::File;
use std::os::raw::c_char;
use ash::{Device, Entry, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{Handle, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};

use log::*;
use simplelog::*;
use winit::platform::unix::WindowExtUnix;
use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

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

    let entry = unsafe {ash::Entry::load().unwrap() };

    let enginename = std::ffi::CString::new(EngineName).unwrap();
    let appname = std::ffi::CString::new(AppName).unwrap();


    let app_info = vk::ApplicationInfo::builder()
        .application_name(&appname)
        .engine_name(&enginename)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::API_VERSION_1_1)
        .engine_version(vk::make_version(0, 1, 0))
        .build();

    let mut extension_name_pointers : Vec<*const c_char> =
        ash_window::enumerate_required_extensions(&window).unwrap()
            .iter()
            .map(|&name| name.as_ptr())
            .collect();

    let layer_names: Vec<std::ffi::CString> =
        vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    // let extension_name_pointers: Vec<*const i8> =
    //     vec![ash::extensions::ext::DebugUtils::name().as_ptr()];
    extension_name_pointers.push(
        ash::extensions::ext::DebugUtils::name().as_ptr());


    let mut debugcreateinfo = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(vulkan_debug_utils_callback))
        .build();

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .push_next(&mut debugcreateinfo)
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers).build();

    let instance = InstanceSafe::new(&entry, &instance_create_info);

    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance.inner);

    let utils_messenger =
        unsafe { debug_utils.create_debug_utils_messenger(&debugcreateinfo, None).unwrap() };

    let (physical_device, physical_device_properties) = GetDefaultPhysicalDevice(&instance);
    let qfamindices = GetGraphicQueue(&instance, &physical_device);

    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfamindices.0)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfamindices.1)
            .queue_priorities(&priorities)
            .build(),
    ];

    let device_extension_name_pointers : Vec<*const i8> = 
        vec![ash::extensions::khr::Swapchain::name().as_ptr()];


    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers)
        .enabled_layer_names(&layer_name_pointers);
    let logical_device =
        unsafe { instance.inner.create_device(physical_device, &device_create_info, None).unwrap()};
    let graphics_queue = unsafe { logical_device.get_device_queue(qfamindices.0, 0) };
    let transfer_queue = unsafe { logical_device.get_device_queue(qfamindices.1, 0) };

    let surface = SurfaceSafe::new(&window, &instance, &entry);

    let swapchain = SwapchainSafe::new(
        &surface,
        physical_device,
        qfamindices.clone(),
        &logical_device,
        &instance);

    let swapchain_images = unsafe {
        swapchain.loader.get_swapchain_images(swapchain.inner).unwrap()
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
            logical_device.create_image_view(&imageview_create_info, None).unwrap()
        };
        swapchain_imageviews.push(imageview);
    }

    unsafe {
        for iv in &swapchain_imageviews {
            logical_device.destroy_image_view(*iv, None);
        }
        logical_device.destroy_device(None);
        debug_utils.destroy_debug_utils_messenger(utils_messenger, None);
    };
}

fn GetGraphicQueue(instance: &InstanceSafe, physical_device: &PhysicalDevice) -> (u32, u32) {
    let queuefamilyproperties =
        unsafe { instance.inner.get_physical_device_queue_family_properties(physical_device.clone()) };
    // dbg!(&queuefamilyproperties);

    let mut found_graphics_q_index = None;
    let mut found_transfer_q_index = None;
    for (index, qfam) in queuefamilyproperties.iter().enumerate() {
        if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            found_graphics_q_index = Some(index as u32);
            info!("Found graphic queue!");
        }
        if qfam.queue_count > 0 && qfam.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            if found_transfer_q_index.is_none()
                || !qfam.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                found_transfer_q_index = Some(index as u32);
                info!("Found transfer queue!");
            }
        }
    }
    (
        found_graphics_q_index.unwrap(),
        found_transfer_q_index.unwrap(),
    )
}

fn GetDefaultPhysicalDevice(instance: &InstanceSafe) -> (PhysicalDevice, PhysicalDeviceProperties) {
    let phys_devs = unsafe { instance.inner.enumerate_physical_devices().unwrap() };

    let mut chosen = None;
    for p in phys_devs {
        let properties = unsafe { instance.inner.get_physical_device_properties(p) };

        let name = String::from(
            unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) }
                .to_str()
                .unwrap(),
        );
        info!("Vulkan device: {}", name);
        if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
            chosen = Some((p, properties));
            info!("Selected device: {}", name);
        }
    }
    chosen.unwrap()
}


unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    let severity = format!("{:?}", message_severity).to_lowercase();
    let ty = format!("{:?}", message_type).to_lowercase();
    if severity == "info" || severity == "verbose" {
        debug!("[{}] {:?}", ty, message);
    } else {
        error!("[{}][{}] {:?}", severity, ty, message);
    }
    vk::FALSE
}


#[repr(transparent)]
pub struct InstanceSafe {
    inner : ash::Instance
}


impl InstanceSafe {
    pub fn new(
            entry : &ash::Entry,
            instance_create_info : &vk::InstanceCreateInfo) -> InstanceSafe {
        let instance_res =  unsafe {
            entry.create_instance(&instance_create_info, None)
        };
        Self {
            inner : instance_res.unwrap()
        }
    }
}

impl Drop for InstanceSafe {
    fn drop(&mut self) {
        unsafe {
            self.inner.destroy_instance(None);
        }
    }
}


pub struct SurfaceSafe {
    inner : SurfaceKHR,
    loader : Surface
}

impl SurfaceSafe {
    pub fn new(window : &Window, instance : &InstanceSafe, entry : &Entry) -> Self {
        let x11_display = window.xlib_display().unwrap();
        let x11_window = window.xlib_window().unwrap();
        let x11_create_info = vk::XlibSurfaceCreateInfoKHR::builder()
            .window(x11_window)
            .dpy(x11_display as *mut vk::Display);
        let xlib_surface_loader = ash::extensions::khr::XlibSurface::new(&entry, &instance.inner);
        let surface = unsafe { xlib_surface_loader.create_xlib_surface(&x11_create_info, None) }.unwrap();
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance.inner);

        Self {
            inner : surface,
            loader : surface_loader
        }
    }
}

impl Drop for SurfaceSafe {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.inner, None);
        }
    }
}

pub struct SwapchainSafe {
    inner : SwapchainKHR,
    loader : Swapchain
}

impl SwapchainSafe {
    pub fn new(
        surface : &SurfaceSafe,
        physical_device : PhysicalDevice,
        qfamindices : (u32, u32),
        logical_device : &Device,
        instance : &InstanceSafe) -> Self {
        let surface_capabilities = unsafe {
            surface.loader.get_physical_device_surface_capabilities(
                physical_device, surface.inner).unwrap()
        };
        let surface_present_modes = unsafe {
            surface.loader.get_physical_device_surface_present_modes(
                physical_device, surface.inner).unwrap()
        };
        let surface_formats = unsafe {
            surface.loader.get_physical_device_surface_formats(
                physical_device, surface.inner).unwrap()
        };

        info!("Creating swapchain!");
        let queuefamilies = [qfamindices.0];
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.inner)
            .min_image_count(
                3.max(surface_capabilities.min_image_count)
                    .min(surface_capabilities.max_image_count)
            )
            .image_format(surface_formats.first().unwrap().format)
            .image_color_space(surface_formats.first().unwrap().color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .build();
        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance.inner, &logical_device);
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&swapchain_create_info, None).unwrap()
        };
        debug!("{:#?}", swapchain_create_info);

        Self {
            inner : swapchain,
            loader : swapchain_loader
        }
    }
}

impl Drop for SwapchainSafe {
    fn drop(&mut self) {
        unsafe {
            // self.loader.destroy_swapchain(self.inner, None);
        }
    }
}