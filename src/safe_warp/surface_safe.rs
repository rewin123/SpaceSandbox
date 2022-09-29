use ash::{prelude::VkResult};
use crate::*;


pub struct SurfaceSafe {
    pub inner : SurfaceKHR,
    pub loader : Surface,
    instance : Arc<InstanceSafe>
}

impl SurfaceSafe {
    pub(crate) fn get_formats(&self, p0: PhysicalDevice) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.loader.get_physical_device_surface_formats(p0, self.inner)
        }
    }
}


impl SurfaceSafe {

    

    pub fn new(window : &Window, instance : &Arc<InstanceSafe>, entry : &Entry) -> Self {

        let surface = unsafe {  ash_window::create_surface(
            entry, &instance.inner, &window, None)
        }.unwrap();
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance.inner);

        Self {
            inner : surface,
            loader : surface_loader,
            instance : instance.clone()
        }
    }
}

impl Drop for SurfaceSafe {
    fn drop(&mut self) {
        info!("Destroy surface");
        unsafe {
            self.loader.destroy_surface(self.inner, None);
        }
    }
}