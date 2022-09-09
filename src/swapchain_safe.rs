use ash::vk;
use ash::vk::{Framebuffer, Image, ImageView};
use log::info;
use crate::*;

pub struct SwapchainSafe {
    pub inner : SwapchainKHR,
    pub loader : Swapchain,
    device : Arc<DeviceSafe>,
    surface : Arc<SurfaceSafe>,
    images : Vec<Image>,
    imageviews: Vec<ImageView>,
    framebuffers : Vec<Framebuffer>,
    extent: vk::Extent2D,
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
        let extent = surface_capabilities.current_extent;

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

        let swapchain_images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain).unwrap()
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

        Self {
            inner : swapchain,
            loader : swapchain_loader,
            device : logical_device.clone(),
            surface : surface.clone(),
            images : swapchain_images,
            imageviews : swapchain_imageviews,
            framebuffers : vec![],
            extent
        }
    }

    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(self.extent.width)
                .height(self.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            self.framebuffers.push(fb);
        }
        Ok(())
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
            for fb in &self.framebuffers {
                self.device.destroy_framebuffer(*fb, None);
            }
            for iv in &self.imageviews {
                self.device.destroy_image_view(*iv, None);
            }
            self.loader.destroy_swapchain(self.inner, None);
        }
    }
}