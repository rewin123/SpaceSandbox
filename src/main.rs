use ash::prelude::VkResult;
use ash::vk;
use winit::window::CursorIcon::Default;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

fn main() {
    let entry = unsafe {ash::Entry::load().unwrap() };

    let enginename = std::ffi::CString::new(EngineName).unwrap();
    let appname = std::ffi::CString::new(AppName).unwrap();


    let app_info = vk::ApplicationInfo::builder()
        .application_name(&appname)
        .engine_name(&enginename)
        .application_version(vk::make_version(0, 1, 0))
        .api_version(vk::make_version(1, 0, 106))
        .engine_version(vk::make_version(0, 1, 0))
        .build();
    let layer_names: Vec<std::ffi::CString> =
        vec![std::ffi::CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layer_name_pointers: Vec<*const i8> = layer_names
        .iter()
        .map(|layer_name| layer_name.as_ptr())
        .collect();
    let extension_name_pointers: Vec<*const i8> =
        vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layer_name_pointers)
        .enabled_extension_names(&extension_name_pointers)
        .build();

    dbg!(&instance_create_info);

    let instance = InstanceSafe::new(&entry, &instance_create_info);

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
    println!("[Debug][{}][{}] {:?}", severity, ty, message);
    vk::FALSE
}



struct InstanceSafe {
    instance : ash::Instance
}

impl InstanceSafe {
    pub fn new(
            entry : &ash::Entry,
            instance_create_info : &vk::InstanceCreateInfo) -> InstanceSafe {
        let instance_res =  unsafe {
            entry.create_instance(&instance_create_info, None)
        };
        Self {
            instance : instance_res.unwrap()
        }
    }
}

impl Drop for InstanceSafe {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
