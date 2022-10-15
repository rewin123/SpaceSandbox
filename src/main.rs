use std::iter;
use std::ops::Deref;
use std::sync::Arc;

use SpaceSandbox::light::PointLight;
use SpaceSandbox::task_server::TaskServer;
use SpaceSandbox::wgpu_light_fill::PointLightPipeline;
use SpaceSandbox::wgpu_texture_present::TexturePresent;
use egui::epaint::ahash::HashMap;
use space_shaders::*;
use specs::*;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use SpaceSandbox::asset_server::AssetServer;
use SpaceSandbox::{GMesh, init_logger, Material, RenderBase, TextureBundle};
use encase::{ShaderType, UniformBuffer};
use SpaceSandbox::pipelines::wgpu_gbuffer_fill::GBufferFill;
use SpaceSandbox::wgpu_gbuffer_fill::{GFramebuffer};

use nalgebra as na;

async fn run() {
    init_logger();
    

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
                        WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                            state.input_system.process_event(input);
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

struct State {
    surface : wgpu::Surface,
    render : Arc<RenderBase>,
    config : wgpu::SurfaceConfiguration,
    size : winit::dpi::PhysicalSize<u32>,
    scene : World,
    camera : Camera,
    camera_buffer : wgpu::Buffer,
    gbuffer_pipeline : GBufferFill,
    light_pipeline : PointLightPipeline,
    light_buffer : TextureBundle,
    gbuffer : GFramebuffer,
    present : TexturePresent,
    point_lights : Vec<PointLight>,
    input_system : InputSystem,
    assets : AssetServer
}

#[derive(ShaderType)]
struct CameraUniform {
    pub view : nalgebra::Matrix4<f32>,
    pub proj : nalgebra::Matrix4<f32>,
    pub pos : nalgebra::Vector3<f32>
}

struct Camera {
    pub pos : nalgebra::Point3<f32>,
    pub frw : nalgebra::Vector3<f32>,
    pub up : nalgebra::Vector3<f32>
}

impl Camera {
    pub fn get_right(&self) -> na::Vector3<f32> {
        self.frw.cross(&self.up)
    }
}


#[derive(Default)]
struct InputSystem {
    key_state : HashMap<winit::event::VirtualKeyCode, bool>
}

impl InputSystem {
    pub fn process_event(&mut self, input : &KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            // log::info!("New {:?} state {:?}", &key, &input.state);
            self.key_state.insert(key, input.state == ElementState::Pressed);
        }
    }

    fn get_key_state(&self, key : VirtualKeyCode) -> bool {
        if let Some(state) = self.key_state.get(&key) {
            *state
        } else {
            false
        }
    }
}


impl Default for Camera {
    fn default() -> Self {
        Self {
            pos : [-3.0, 9.0, 0.0].into(),
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
            proj,
            pos : na::Vector3::new(self.pos.x, self.pos.y, self.pos.z)
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

        let extent = wgpu::Extent3d {
            width : config.width,
            height : config.height,
            depth_or_array_layers : 1
        };

        let mut world = World::new();
        world.register::<GMesh>();
        world.register::<Material>();

        let task_server = Arc::new(TaskServer::new());

        let render = Arc::new(RenderBase {
            device,
            queue,
        });

        let mut assets = AssetServer::new(&render, &task_server);
        

        let scene = assets.wgpu_gltf_load(
            &render.device,
            "res/test_res/models/sponza/glTF/Sponza.gltf".into(),
            &mut world);

        let gbuffer = GBufferFill::new(
            &render.device,
            &camera_buffer,
            config.format,
            wgpu::Extent3d {
                width : config.width,
                height : config.height,
                depth_or_array_layers : 1
            });

        let framebuffer = GBufferFill::spawn_framebuffer(
            &render.device,
            extent);

        let present = TexturePresent::new(
            &render.device, 
            config.format, 
            wgpu::Extent3d {
                width : config.width,
                height : config.height,
                depth_or_array_layers : 1
            });

        let lights = vec![
            PointLight::new(&render.device, [0.0, 10.0, 0.0].into())
        ];

        let light_pipeline = PointLightPipeline::new(&render.device, &camera_buffer, extent);
        let light_buffer = light_pipeline.spawn_framebuffer(&render.device, extent);

        Self {
            surface,
            config,
            size,
            scene : world,
            camera : Camera::default(),
            camera_buffer,
            gbuffer_pipeline : gbuffer,
            gbuffer : framebuffer,
            present,
            point_lights : lights,
            light_pipeline,
            light_buffer,
            input_system : InputSystem::default(),
            assets,
            render
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.render.device, &self.config);

            let size = wgpu::Extent3d {
                width : self.config.width,
                height : self.config.height,
                depth_or_array_layers : 1
            };

            self.gbuffer_pipeline = GBufferFill::new(
                &self.render.device,
                &self.camera_buffer,
                self.config.format,
                size.clone()
            );

            self.present = TexturePresent::new(
                &self.render.device, 
                self.config.format, 
                size);

            self.light_pipeline = PointLightPipeline::new(
                &self.render.device,
                &self.camera_buffer,
                size
            );

            self.light_buffer = self.light_pipeline.spawn_framebuffer(&self.render.device, size);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let speed = 0.1;
        if self.input_system.get_key_state(VirtualKeyCode::W) {
            self.camera.pos += self.camera.frw * speed;
        } 
        if self.input_system.get_key_state(VirtualKeyCode::S) {
            self.camera.pos -= self.camera.frw * speed;
        }
        if self.input_system.get_key_state(VirtualKeyCode::D) {
            self.camera.pos += self.camera.get_right() * speed;
        }
        if self.input_system.get_key_state(VirtualKeyCode::A) {
            self.camera.pos -= self.camera.get_right() * speed;
        }

        let camera_unifrom = self.camera.build_uniform();
        let mut uniform = encase::UniformBuffer::new(vec![]);
        uniform.write(&camera_unifrom).unwrap();
        let inner = uniform.into_inner();

        let tmp_buffer = self.render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &inner,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let mut encoder = self
        .render.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Update encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &tmp_buffer, 
            0, 
            &self.camera_buffer, 
            0, 
            inner.len() as wgpu::BufferAddress);
        self.render.queue.submit(iter::once(encoder.finish()));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
        .render.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            self.gbuffer_pipeline.draw(&mut encoder, &self.scene, &self.gbuffer);
        }

        self.light_pipeline.draw(&self.render.device, &mut encoder, &self.point_lights, &self.light_buffer, &self.gbuffer);

        self.present.draw(&self.render.device, &mut encoder, &self.light_buffer, &view);

        self.render.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}



fn main() {
    pollster::block_on(run());
}