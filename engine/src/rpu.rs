use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::Version;
use vulkano::device::physical::{PhysicalDevice, QueueFamily, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features, QueuesIter, Queue};
use vulkano::render_pass::Framebuffer;
use vulkano::swapchain::{Surface, Swapchain};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use std::sync::Arc;
use vulkano::image::*;


fn select_surfaced_physical_device<'a>(
    instance: &'a Arc<Instance>,
    surface: Arc<Surface<Window>>,
    device_extensions: &DeviceExtensions,
) -> (PhysicalDevice<'a>, QueueFamily<'a>) {
    let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
        .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
        .filter_map(|p| {
            p.queue_families()
                .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                .map(|q| (p, q))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
        })
        .expect("no device available");

    (physical_device, queue_family)
}


pub struct RPU {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue : Arc<Queue>
}

pub struct WinRpu {
    pub rpu : RPU,
    pub surface : Arc<Surface<Window>>,
    pub swapchain : Arc<Swapchain<Window>>, 
    pub swapchain_images : Vec<Arc<SwapchainImage<Window>>>,
    pub framebuffers : Vec<Arc<Framebuffer>>
}

impl WinRpu {
    fn default() -> (Self, EventLoop<()>) {
        let required_extensions = vulkano_win::required_extensions();
        let instance = 
                Instance::new(
                    None, 
                    Version::V1_1, 
                        &required_extensions, 
                        None).expect("Failed to create Instance");
        
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let (physical_device, queue_family) = 
            select_surfaced_physical_device(
                &instance, surface.clone(), &device_extensions);

        let (device, mut queues) = {
            Device::new(
                physical_device,
                &Features::none(),
                &physical_device
                    .required_extensions()
                    .union(&device_extensions), // new
                [(queue_family, 0.5)].iter().cloned(),
            )
            .expect("failed to create device")
        };

        let queue = queues.next().unwrap();

        let (mut swapchain, images) = {
            let caps = surface
                .capabilities(physical_device)
                .expect("failed to get surface capabilities");
    
            let dimensions: [u32; 2] = surface.window().inner_size().into();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
    
            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count + 1)
                .format(format)
                .dimensions(dimensions)
                .usage(ImageUsage::color_attachment())
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .build()
                .expect("failed to create swapchain")
        };

        (Self {
            rpu : RPU {
                device,
                instance,
                queue,
            },
            swapchain,
            surface,
            swapchain_images : images
        }, event_loop)
    }
}

impl RPU {
    pub fn create_image(
            &self, width : u32, 
            height : u32, 
            format : vulkano::format::Format) 
                -> Result<Arc<StorageImage>, ImageCreationError> {
        StorageImage::new(
            self.device.clone(),
            ImageDimensions::Dim2d {
                width,
                height,
                array_layers : 1
            },
            format,
            Some(self.queue.family())
        )
    }
}

impl Default for RPU {
    fn default() -> Self {
        let instance = 
            Instance::new(None, Version::V1_1, &InstanceExtensions::none(), None).expect("Failed to create Instance");
        let physical = PhysicalDevice::enumerate(&instance).next().expect("Physical device not found");

        let queues_family = physical.queue_families()
            .find(|&q| q.supports_graphics())
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            Device::new(
                physical, 
                &Features::none(),
                &DeviceExtensions::none(), 
                            [(queues_family, 0.5)].iter().cloned()).expect("Failed to create device")
        };

        let queue = queues.next().expect("Failed to grab first queue");
        
        Self {
            instance : instance.clone(),
            device,
            queue,
        }

    }
}