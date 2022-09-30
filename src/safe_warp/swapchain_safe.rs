use std::borrow::BorrowMut;
use ash::vk;
use ash::vk::{Framebuffer, Image, ImageView};
use log::info;
use crate::*;
use std::default::Default;
use std::pin::Pin;
use gpu_allocator::vulkan::AllocationCreateDesc;
use egui_winit_ash_integration::MemoryLocation;

pub struct SwapchainSafe {
    pub inner : SwapchainKHR,
    pub loader : Swapchain,
    device : Arc<DeviceSafe>,
    pub surface : Arc<SurfaceSafe>,
    pub images : Vec<Image>,
    pub imageviews: Vec<ImageView>,
    pub framebuffers : Vec<Framebuffer>,
    pub extent: vk::Extent2D,
    pub image_available: Vec<vk::Semaphore>,
    pub rendering_finished: Vec<vk::Semaphore>,
    pub may_begin_drawing: Vec<vk::Fence>,
    pub amount_of_images: u32,
    pub current_image: usize,

    pub depth_image: vk::Image,
    pub depth_image_allocation: Option<gpu_allocator::vulkan::Allocation>,
    pub depth_imageview: vk::ImageView,

    pub allocator : Arc<AllocatorSafe>,

    pub format : vk::SurfaceFormatKHR
}

impl SwapchainSafe {
    pub fn new(
        surface : &Arc<SurfaceSafe>,
        physical_device : PhysicalDevice,
        qfamindices : &QueueFamilies,
        logical_device : &Arc<DeviceSafe>,
        instance : &InstanceSafe,
        allocator: Arc<AllocatorSafe>) -> Self {
        let surface_capabilities = unsafe {
            surface.loader.get_physical_device_surface_capabilities(
                physical_device, surface.inner).unwrap()
        };
        let extent = surface_capabilities.current_extent;

        let surface_formats = unsafe {
            surface.loader.get_physical_device_surface_formats(
                physical_device, surface.inner).unwrap()
        };

        let format = surface_formats.first().unwrap().clone();

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
            .present_mode(vk::PresentModeKHR::IMMEDIATE)
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

        let amount_of_images = swapchain_images.len() as u32;
        let mut image_available = vec![];
        let mut rendering_finished = vec![];
        let semaphoreinfo = vk::SemaphoreCreateInfo::builder();
        let fenceinfo = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
        let mut may_begin_drawing = vec![];
        for _ in 0..amount_of_images {
            let semaphore_available =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }.unwrap();
            let semaphore_finished =
                unsafe { logical_device.create_semaphore(&semaphoreinfo, None) }.unwrap();
            image_available.push(semaphore_available);
            rendering_finished.push(semaphore_finished);
            let fence = unsafe { logical_device.create_fence(&fenceinfo, None) }.unwrap();
            may_begin_drawing.push(fence);
        }

        let extent3d = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };
        let depth_image_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .format(vk::Format::D32_SFLOAT)
            .extent(extent3d)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&queuefamilies);

        let depth_image = unsafe {
            logical_device.create_image(&depth_image_info, None).unwrap()
        };

        let allocation_info = unsafe {
            AllocationCreateDesc {
                name: "depth allocation",
                requirements: logical_device.get_image_memory_requirements(depth_image),
                location: gpu_allocator::MemoryLocation::GpuOnly,
                linear: false
            }
        };

        let depth_image_allocation = allocator.allocate(&allocation_info).unwrap();

        unsafe {
            logical_device.bind_image_memory(
                depth_image,
                depth_image_allocation.memory(),
                depth_image_allocation.offset()).unwrap();
        }
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::DEPTH)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);
        let imageview_create_info = vk::ImageViewCreateInfo::builder()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::D32_SFLOAT)
            .subresource_range(*subresource_range);
        let depth_imageview =
            unsafe { logical_device.create_image_view(&imageview_create_info, None) }.unwrap();

        Self {
            inner : swapchain,
            loader : swapchain_loader,
            device : logical_device.clone(),
            surface : surface.clone(),
            images : swapchain_images,
            imageviews : swapchain_imageviews,
            framebuffers : vec![],
            extent,
            amount_of_images,
            current_image:0,
            image_available,
            rendering_finished,
            may_begin_drawing,
            depth_image,
            depth_imageview,
            depth_image_allocation : Some(depth_image_allocation),
            allocator : allocator.clone(),
            format
        }
    }

    pub fn create_framebuffers(
        &mut self,
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
    ) -> Result<(), vk::Result> {
        for iv in &self.imageviews {
            let iview = [*iv, self.depth_imageview];
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
        info!("Destroying swapchain");
        unsafe {
            self.device.device_wait_idle().expect("Waiting problem");
            self.device.destroy_image_view(self.depth_imageview, None);

            self.allocator.free(self.depth_image_allocation.take().unwrap());
            self.allocator.device.destroy_image(self.depth_image, None);

            for semaphore in &self.image_available {
                self.device.destroy_semaphore(*semaphore, None);
            }
            for semaphore in &self.rendering_finished {
                self.device.destroy_semaphore(*semaphore, None);
            }
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