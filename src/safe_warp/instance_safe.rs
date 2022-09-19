use ash::{Instance, vk};
use log::info;

pub struct InstanceSafe {
    pub inner : ash::Instance
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

impl std::ops::Deref for InstanceSafe {
    type Target = Instance;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for InstanceSafe {
    fn drop(&mut self) {
        info!("Destroy instance");
        unsafe {
            self.inner.destroy_instance(None);
        }
    }
}