use std::iter;
use std::ops::Deref;
use std::sync::Arc;

use SpaceSandbox::ui::{Gui, FpsCounter};
use bytemuck::{Zeroable, Pod};
use egui::epaint::ahash::HashMap;
use egui_gizmo::GizmoMode;
use egui_wgpu_backend::ScreenDescriptor;
use space_shaders::*;
use specs::*;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use SpaceSandbox::{init_logger};
use encase::{ShaderType, UniformBuffer};
use space_assets::*;

use nalgebra as na;
use space_core::{RenderBase, TaskServer};
use space_render::{pipelines::*, Camera};
use space_render::light::*;
use space_render::pipelines::wgpu_ssao::SSAO;

#[repr(C)]
#[derive(Clone, Zeroable, Pod, Copy)]
pub struct SmoothUniform {
    size : [i32; 2]
}

impl TextureTransformUniform for SmoothUniform {
    fn get_bytes(&self) -> Vec<u8> {
        let bytes = bytemuck::bytes_of(self);
        bytes.to_vec()
    }
}

async fn run() {
    init_logger();
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();
    

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    window.set_title("Space sandbox");

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;

    event_loop.run(move |event, _, control_flow| {
        state.gui.platform.handle_event(&event);
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
                match state.render(&window) {
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
    light_shadow : PointLightShadowPipeline,
    ssao_pipeline : SSAO,
    ssao_framebuffer : CommonFramebuffer,

    ssao_smooth_pipeline : TextureTransformPipeline,
    ssao_smooth_framebuffer : CommonFramebuffer,

    light_pipeline : PointLightPipeline,
    gamma_correction : TextureTransformPipeline,
    light_buffer : TextureBundle,
    gbuffer : GFramebuffer,
    present : TexturePresent,
    gamma_buffer : CommonFramebuffer,
    point_lights : Vec<PointLight>,
    input_system : InputSystem,
    assets : AssetServer,
    gui : Gui,
    fps : FpsCounter,
    device_name : String
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
                features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
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
            present_mode: wgpu::PresentMode::Immediate,
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

        assets.wgpu_gltf_load(
            &render.device,
            "res/test_res/models/sponza/glTF/Sponza.gltf".into(),
            &mut world);

        let gbuffer = GBufferFill::new(
            &render,
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

        let mut lights = vec![
            PointLight::new(&render, [100.0, 100.0, 0.0].into(), true),
            PointLight::new(&render, [-100.0, 100.0, 0.0].into(), true),
        ];
        lights[0].intensity = 1000.0;
        lights[1].intensity = 1000.0;

        let point_light_shadow = PointLightShadowPipeline::new(&render);

        let light_pipeline = PointLightPipeline::new(&render, &camera_buffer, extent);
        let light_buffer = light_pipeline.spawn_framebuffer(&render.device, extent);

        let gui = Gui::new(
            &render, 
            config.format, 
            extent, 
            window.scale_factor());

        let fps = FpsCounter::default();

        let mut gamma_correction = TextureTransformPipeline::new(
            &render,
            wgpu::TextureFormat::Rgba32Float,
            extent,
            1,
            1,
            None,
            include_str!("../shaders/wgsl/gamma_correction.wgsl").into()
        );

        let gamma_buffer = gamma_correction.spawn_framebuffer();

        let ssao_pipeline = SSAO::new(
            &render,
            wgpu::TextureFormat::R8Unorm,
            wgpu::Extent3d {
                width : extent.width / 2,
                height : extent.height / 2,
                depth_or_array_layers : 1
            },
            1,
            1,
            include_str!("../shaders/wgsl/ssao.wgsl").into()
        );

        let ssao_buffer = ssao_pipeline.spawn_framebuffer();

        let size_uniform = SmoothUniform {
            size : [extent.width as i32 / 2, extent.height as i32 / 2]
        };

        let ssao_smooth_pipeline = TextureTransformPipeline::new(
            &render, 
            wgpu::TextureFormat::R8Unorm, 
            wgpu::Extent3d {
                width : extent.width / 2,
                height : extent.height / 2,
                depth_or_array_layers : 1
            }, 
            1, 
            1, 
            Some(Box::new(size_uniform)), 
            include_str!("../shaders/wgsl/smooth.wgsl").into());

        let ssao_smooth_framebuffer = ssao_smooth_pipeline.spawn_framebuffer();

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
            render,
            gui,
            fps,
            light_shadow : point_light_shadow,
            gamma_correction,
            gamma_buffer,
            ssao_pipeline,
            ssao_framebuffer : ssao_buffer,
            device_name : adapter.get_info().name.clone(),
            ssao_smooth_pipeline,
            ssao_smooth_framebuffer
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
                &self.render,
                &self.camera_buffer,
                self.config.format,
                size.clone()
            );

            self.gbuffer = GBufferFill::spawn_framebuffer(
                &self.render.device,
            size.clone());

            self.present = TexturePresent::new(
                &self.render.device, 
                self.config.format, 
                size);

            self.light_pipeline = PointLightPipeline::new(
                &self.render,
                &self.camera_buffer,
                size
            );

            self.gamma_correction = TextureTransformPipeline::new(
                &self.render,
                wgpu::TextureFormat::Rgba32Float,
                size,
                1,
                1,
                None,
                include_str!("../shaders/wgsl/gamma_correction.wgsl").into()
            );

            self.ssao_pipeline = SSAO::new(
                &self.render, 
                wgpu::TextureFormat::R8Unorm, 
                wgpu::Extent3d {
                    width : size.width / 2,
                    height : size.height / 2,
                    depth_or_array_layers : 1
                }, 
                1, 
                1, 
                include_str!("../shaders/wgsl/ssao.wgsl").into()
            );

            self.gamma_buffer = self.gamma_correction.spawn_framebuffer();

            self.light_buffer = self.light_pipeline.spawn_framebuffer(&self.render.device, size);

            self.ssao_framebuffer = self.ssao_pipeline.spawn_framebuffer();

            let size_uniform = SmoothUniform {
                size : [size.width as i32 / 2, size.height as i32 / 2]
            };

            self.ssao_smooth_pipeline = TextureTransformPipeline::new(
                &self.render, 
                wgpu::TextureFormat::R8Unorm, 
                wgpu::Extent3d {
                    width : size.width / 2,
                    height : size.height / 2,
                    depth_or_array_layers : 1
                }, 
                1, 
                1, 
                Some(Box::new(size_uniform)), 
                include_str!("../shaders/wgsl/smooth.wgsl").into());
    
            self.ssao_smooth_framebuffer = self.ssao_smooth_pipeline.spawn_framebuffer();
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        let speed = 0.3;
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
        if self.input_system.get_key_state(VirtualKeyCode::Space) {
            self.camera.pos += self.camera.up  * speed;
        }
        if self.input_system.get_key_state(VirtualKeyCode::LShift) {
            self.camera.pos -= self.camera.up * speed;
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

    fn render(&mut self, window : &Window) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.ssao_pipeline.update(&self.camera);
        for light in &mut self.point_lights {
            light.update_buffer(&self.render);
        }
        self.render.device.poll(wgpu::Maintain::Wait);

        let mut encoder = self
        .render.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.gbuffer_pipeline.draw(&self.assets,&mut encoder, &self.scene, &self.gbuffer);
        self.light_shadow.draw(&mut encoder, &mut self.point_lights, &self.scene);
        self.ssao_pipeline.draw(&mut encoder, &self.gbuffer, &self.ssao_framebuffer.dst[0]);
        self.ssao_smooth_pipeline.draw(&self.render.device, &mut encoder, &[&self.ssao_framebuffer.dst[0]], &[&self.ssao_smooth_framebuffer.dst[0]]);
        self.light_pipeline.draw(&self.render.device, &mut encoder, &self.point_lights, &self.light_buffer, &self.gbuffer, &self.ssao_smooth_framebuffer.dst[0]);
        self.gamma_correction.draw(&self.render.device, &mut encoder, &[&self.light_buffer], &[&self.gamma_buffer.dst[0]]);

        self.present.draw(&self.render.device, &mut encoder, &self.gamma_buffer.dst[0], &view);
        // self.present.draw(&self.render.device, &mut encoder, &self.ssao_smooth_framebuffer.dst[0], &view);

        self.gui.begin_frame();

        egui::TopBottomPanel::top("top_panel").show(
            &self.gui.platform.context(), |ui| {

                ui.horizontal(|ui| {
                    self.fps.draw(ui);
                    ui.label(&self.device_name);

                    ui.add(egui::Slider::new(&mut self.ssao_pipeline.scale, 0.0..=1000.0));
                });

        });

        let gui_output = self.gui.end_frame(Some(window));
        self.gui.draw(gui_output, 
            ScreenDescriptor {
                physical_width: self.config.width,
                physical_height: self.config.height,
                scale_factor: window.scale_factor() as f32,
            }, 
            &mut encoder, 
            &view);

        self.render.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.assets.sync_tick();

        Ok(())
    }
}



fn main() {
    pollster::block_on(run());
}