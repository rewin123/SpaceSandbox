use std::sync::Arc;
use bevy::prelude::{info, Resource};
use space_core::RenderBase;

#[derive(Resource)]
pub struct ApiBase {
    pub instance : wgpu::Instance,
    pub adapter : wgpu::Adapter,
    pub surface : wgpu::Surface,
    pub config : wgpu::SurfaceConfiguration,
    pub render_base : Arc<RenderBase>,
    pub size : winit::dpi::PhysicalSize<u32>
}

impl ApiBase {
    pub fn new(window : &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        let size = window.inner_size();
        let surface = unsafe {
            instance.create_surface(window)
        };
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference : wgpu::PowerPreference::HighPerformance,
                compatible_surface : Some(&surface),
                force_fallback_adapter: false
            }
        )).unwrap();

        println!("Device: {:?}", &adapter.get_info().name);
        

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
                    | wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                limits : wgpu::Limits::default(),
                label: None
            },
            None
        )).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Immediate,
            alpha_mode: wgpu::CompositeAlphaMode::Auto
        };
        surface.configure(&device, &config);


        Self {
            instance,
            adapter,
            surface,
            config,
            size,
            render_base : Arc::new(RenderBase {
                device,
                queue
            })
        }
    }
}