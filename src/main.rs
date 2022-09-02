use std::fs::File;
use std::os::raw::c_char;
use ash::{vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::vk::{PhysicalDevice, PhysicalDeviceProperties};

use log::*;
use simplelog::*;

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

    dbg!(&instance_create_info);

    let instance = InstanceSafe::new(&entry, &instance_create_info);

    let surface_loader = unsafe {
        ash_window::create_surface(
            &entry,
            &instance.inner,
            &window, None).unwrap()
    };

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
    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_layer_names(&layer_name_pointers);
    let logical_device =
        unsafe { instance.inner.create_device(physical_device, &device_create_info, None).unwrap()};
    let graphics_queue = unsafe { logical_device.get_device_queue(qfamindices.0, 0) };
    let transfer_queue = unsafe { logical_device.get_device_queue(qfamindices.1, 0) };

    unsafe {

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
struct InstanceSafe {
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
