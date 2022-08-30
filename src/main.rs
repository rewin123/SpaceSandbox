use ash::InstanceError;
use ash::version::EntryV1_0;
use ash::version::InstanceV1_0;
use ash::vk;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

fn main() {
    let entry = ash::Entry::new().unwrap();
    let instance = InstanceSafe::new(&entry).unwrap();

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
    pub fn new(entry : &ash::Entry) -> Result<Self, InstanceError> {
        let instance_res =  unsafe {
            entry.create_instance(&Default::default(), None)
        };
        match instance_res {
            Ok(res) => {
                return Ok(Self {
                    instance : res
                });
            },
            Err(err) => {
                return Err(err);
            }
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