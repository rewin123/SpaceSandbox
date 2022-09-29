use crate::*;

pub fn get_logical_device(
    layer_names: &Vec<&str>,
    instance: &InstanceSafe,
    physical_device: PhysicalDevice,
    qfamindex : &QueueFamilies) -> (Device, Queues) {


    let priorities = [1.0f32];
    let queue_infos = [
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfamindex.graphics_q_index)
            .queue_priorities(&priorities)
            .build(),
        vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(qfamindex.transfer_q_index)
            .queue_priorities(&priorities)
            .build(),
    ];

    let device_extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::khr::Swapchain::name().as_ptr()];

    let layer_names_c: Vec<std::ffi::CString> = layer_names
        .iter()
        .map(|&ln| std::ffi::CString::new(ln).unwrap())
        .collect();
    let layer_name_pointers: Vec<*const i8> = layer_names_c
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&device_extension_name_pointers)
        .enabled_layer_names(&layer_name_pointers);


    let logical_device = unsafe { instance.create_device(physical_device, &device_create_info, None).unwrap() };

    let graphics_queue = unsafe { logical_device.get_device_queue(qfamindex.graphics_q_index, 0) };
    let transfer_queue = unsafe { logical_device.get_device_queue(qfamindex.transfer_q_index, 0) };

    let queues = Queues {
        graphics_queue,
        transfer_queue
    };

    (logical_device, queues)
}


pub fn init_instance(
    entry : &Entry,
    layer_names: &[&str],
    window : &Window
) -> InstanceSafe {
    let enginename = std::ffi::CString::new(EngineName).unwrap();
    let appname = std::ffi::CString::new(AppName).unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&appname)
        .engine_name(&enginename)
        .application_version(vk::make_api_version(0, 1, 0, 0))
        .api_version(vk::API_VERSION_1_1)
        .engine_version(vk::make_version(0, 1, 0))
        .build();

    let layer_names_c: Vec<std::ffi::CString> = layer_names
        .iter()
        .map(|&ln| std::ffi::CString::new(ln).unwrap())
        .collect();
    let layer_name_pointers: Vec<*const i8> = layer_names_c
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();

    let mut extension_name_pointers : Vec<*const c_char> =
        ash_window::enumerate_required_extensions(&window).unwrap()
            .iter()
            .map(|&name| name.as_ptr())
            .collect();

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
    instance
}

pub fn get_graphic_queue(instance: &InstanceSafe, physical_device: &PhysicalDevice) -> QueueFamilies {
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

    QueueFamilies {
        graphics_q_index : found_graphics_q_index.unwrap(),
        transfer_q_index : found_transfer_q_index.unwrap()
    }
}

pub fn get_default_physical_device(instance: &InstanceSafe) -> (PhysicalDevice, PhysicalDeviceProperties) {
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