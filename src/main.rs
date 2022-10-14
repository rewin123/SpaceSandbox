use std::iter;
use std::ops::Add;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use SpaceSandbox::asset_server::AssetServer;
use SpaceSandbox::{GMesh, GVertex};
use encase::{ShaderType, UniformBuffer};
use SpaceSandbox::pipelines::wgpu_gbuffer_fill::GBufferFill;
use SpaceSandbox::wgpu_gbuffer_fill::GFramebuffer;

struct State {
    surface : wgpu::Surface,
    device : wgpu::Device,
    queue : wgpu::Queue,
    config : wgpu::SurfaceConfiguration,
    size : winit::dpi::PhysicalSize<u32>,
    scene : Vec<GMesh>,
    camera : Camera,
    camera_buffer : wgpu::Buffer,
    gbuffer_pipeline : GBufferFill,
    gbuffer : GFramebuffer
}

#[derive(ShaderType)]
struct CameraUniform {
    pub view : nalgebra::Matrix4<f32>,
    pub proj : nalgebra::Matrix4<f32>,
}

struct Camera {
    pub pos : nalgebra::Point3<f32>,
    pub frw : nalgebra::Vector3<f32>,
    pub up : nalgebra::Vector3<f32>
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos : [-3.0, 1.0, 0.0].into(),
            frw : [1.0, 0.0, 0.0].into(),
            up : [0.0, 1.0, 0.0].into()
        }
    }
}

impl Camera {
    fn build_uniform(&self) -> CameraUniform {

        let mut target = self.pos + self.frw;
        let view = nalgebra::Matrix4::look_at_rh(
            &self.pos,
            &target,
            &self.up);
        let proj = nalgebra::Matrix4::<f32>::new_perspective(
            1.0,
            3.14 / 2.0,
            0.01,
            10000.0);
        CameraUniform {
            view,
            proj
        }
    }
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe {
            instance.create_surface(window)
        };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference : wgpu::PowerPreference::HighPerformance,
                compatible_surface : Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None
            },
            None
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);


        let camera = Camera::default();
        let camera_uniform = camera.build_uniform();

        let mut camera_cpu_buffer = UniformBuffer::new(vec![0u8;100]);
        camera_cpu_buffer.write(&camera_uniform);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Camera uniform buffer"),
            contents : &camera_cpu_buffer.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });







        let scene = AssetServer::wgpu_gltf_load(
            &device,
            "res/test_res/models/sponza/glTF/Sponza.gltf".into());

        let gbuffer = GBufferFill::new(
            &device,
            &camera_buffer,
            config.format,
            wgpu::Extent3d {
                width : config.width,
                height : config.height,
                depth_or_array_layers : 1
            });

        let framebuffer = GBufferFill::spawn_framebuffer(
            &device,
            wgpu::Extent3d {
                width : config.width,
                height : config.height,
                depth_or_array_layers : 1
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            scene,
            camera : Camera::default(),
            camera_buffer,
            gbuffer_pipeline : gbuffer,
            gbuffer : framebuffer
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.gbuffer_pipeline = GBufferFill::new(
                &self.device,
                &self.camera_buffer,
                self.config.format,
                wgpu::Extent3d {
                    width : self.config.width,
                    height : self.config.height,
                    depth_or_array_layers : 1
                }
            );
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {

    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            self.gbuffer_pipeline.draw(&mut encoder, &self.scene, &self.gbuffer);
        }

        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Default::default()
                    }
                )],
                depth_stencil_attachment: None
            });
            
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                if !state.input(event) {
                    // UPDATED!
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}


fn main() {
    pollster::block_on(run());
}