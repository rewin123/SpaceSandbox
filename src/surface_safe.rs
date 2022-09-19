use ash::{prelude::VkResult, extensions::khr};
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
        
        // let x11_display = window.xlib_display().unwrap();
        // let x11_window = window.xlib_window().unwrap();
        // let x11_create_info = vk::XlibSurfaceCreateInfoKHR::builder()
        //     .window(x11_window)
        //     .dpy(x11_display as *mut vk::Display);
        // let xlib_surface_loader = ash::extensions::khr::XlibSurface::new(&entry, &instance.inner);
        // let surface = unsafe { xlib_surface_loader.create_xlib_surface(&x11_create_info, None) }.unwrap();
        // let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance.inner);


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