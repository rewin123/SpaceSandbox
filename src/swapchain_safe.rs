use ash::vk;
use log::info;
use crate::*;

pub struct SwapchainSafe {
    pub inner : SwapchainKHR,
    pub loader : Swapchain,
    device : Arc<DeviceSafe>,
    surface : Arc<SurfaceSafe>
}

impl SwapchainSafe {
    pub fn new(
        surface : &Arc<SurfaceSafe>,
        physical_device : PhysicalDevice,
        qfamindices : &QueueFamilies,
        logical_device : &Arc<DeviceSafe>,
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
        let queuefamilies = [qfamindices.graphics_q_index];
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
            loader : swapchain_loader,
            device : logical_device.clone(),
            surface : surface.clone()
        }
    }
}

impl Deref for SwapchainSafe {
    type Target = SwapchainKHR;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Drop for SwapchainSafe {
    fn drop(&mut self) {
        info!("Destroy swapchain");
        unsafe {
            self.loader.destroy_swapchain(self.inner, None);
        }
    }
}