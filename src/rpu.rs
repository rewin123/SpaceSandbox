use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::Version;
use vulkano::device::physical::{PhysicalDevice, QueueFamily, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features, QueuesIter, Queue};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, RenderPass};
use vulkano::swapchain::{Surface, Swapchain};
use vulkano_win::VkSurfaceBuild;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use std::sync::Arc;
use vulkano::{image::*};
use vulkano::format::Format;


#[derive(Debug, Clone)]
pub struct RPU {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue : Arc<Queue>
}

#[derive(Clone)]
pub struct WinRpu {
    pub rpu : RPU,
    pub surface : Arc<Surface<Window>>,
    pub swapchain : Arc<Swapchain<Window>>, 
    pub swapchain_images : Vec<Arc<SwapchainImage<Window>>>,
    pub framebuffers : Vec<Arc<Framebuffer>>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    device : Arc<Device>,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    let depth_buffer = ImageView::new(
        AttachmentImage::transient(device.clone(), dimensions, Format::D16_UNORM).unwrap(),
    )
    .unwrap();

    images
        .iter()
        .map(|image| -> Arc<Framebuffer> {
            let view = ImageView::new(image.clone()).unwrap();
            Framebuffer::start(render_pass.clone())
                .add(view.clone()).unwrap()
                .add(depth_buffer.clone()).unwrap()
                .build().unwrap()
            }
        )
        .collect::<Vec<_>>()
}

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


impl WinRpu {
    pub fn default() -> (Self, EventLoop<()>) {
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

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `color` is a custom name we give to the first and only attachment.
                color: {
                    // `load: Clear` means that we ask the GPU to clear the content of this
                    // attachment at the start of the drawing.
                    load: Clear,
                    // `store: Store` means that we ask the GPU to store the output of the draw
                    // in the actual image. We could also ask it to discard the result.
                    store: Store,
                    // `format: <ty>` indicates the type of the format of the image. This has to
                    // be one of the types of the `vulkano::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: swapchain.format(),
                    // TODO:
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                // We use the attachment named `color` as the one and only color attachment.
                color: [color],
                // No depth-stencil attachment is indicated with empty brackets.
                depth_stencil: {depth}
            }
        )
        .unwrap();

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let framebuffers = 
            window_size_dependent_setup(
                &images, 
                render_pass.clone(), 
                &mut viewport,
                device.clone());

        (Self {
            rpu : RPU {
                device,
                instance,
                queue,
            },
            swapchain,
            surface,
            swapchain_images : images,
            framebuffers,
            render_pass,
            viewport
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