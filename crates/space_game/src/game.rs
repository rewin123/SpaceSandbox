use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::mem::swap;
use std::ops::{DerefMut, Deref};
use std::sync::{Arc, RwLock};
use atomic_refcell::AtomicRefMut;
use egui::color::gamma_from_linear;
use wgpu::{Extent3d, ShaderStages, SurfaceTexture, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use space_assets::{AssetServer, Material};
use space_core::{Camera, RenderBase, TaskServer};
use crate::*;
use encase::*;
use wgpu::util::DeviceExt;
use wgpu_profiler::*;
use space_core::ecs::*;


#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SceneType {
    MainMenu,
    StationBuilding
}

#[derive(Default)]
pub struct PluginBase {
    render_plugin : Vec<Box<dyn RenderPlugin>>,
    scheldue_plugin : Vec<Box<dyn SchedulePlugin>>
}

pub struct EguiContext {
    ctx : egui::Context
}

impl Deref for EguiContext {
    type Target = egui::Context;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for EguiContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

pub struct GameScene {
    pub world : World,
    pub scheduler : Schedule,
    pub camera : Camera,
    pub camera_buffer : wgpu::Buffer,
}


pub struct Game {
    pub api : ApiBase,
    event_loop : Option<winit::event_loop::EventLoop<()>>,
    pub render_base : Arc<RenderBase>,
    pub input : InputSystem,
    plugins : Option<PluginBase>,
    pub render_view : Option<TextureView>,
    pub task_server : Arc<TaskServer>,
    pub scene : GameScene,
    pub commands : Vec<GameCommands>,
    pub is_exit_state : bool
}

fn poll_device( render_base : Res<Arc<RenderBase>>) {
    render_base.device.poll(wgpu::Maintain::Wait);
}

impl Game {

    pub fn clear_plugins(&mut self) {
        self.plugins = Some(PluginBase::default());
    }

    pub fn exec_commands(&mut self) {
        let mut cmds = vec![];
        swap(&mut cmds, &mut self.commands);

        for cmd in cmds {
            match cmd {
                GameCommands::Exit => {
                    self.is_exit_state = true;
                }
                GameCommands::AbstractChange(func) => {
                    func(self);
                }
            }
        }
    }

    pub fn get_default_material(&mut self) -> Material {
        let mut assets_ref = self.get_assets();
        let mut assets = assets_ref.deref_mut();
        Material {
            color: assets.new_asset(assets.default_color.clone()),
            normal: assets.new_asset(assets.default_normal.clone()),
            metallic_roughness: assets.new_asset(assets.default_color.clone()),
            version_sum: 0,
            gbuffer_bind: None
        }
    }

    pub fn get_assets(&mut self) -> Mut<AssetServer> {
        self.scene.world.get_resource_mut::<AssetServer>().unwrap()
    }

    pub fn add_render_plugin<T>(&mut self, plugin : T)
        where T : RenderPlugin + 'static {
        let mut plugins = self.plugins.take().unwrap();
        plugins.render_plugin.push(Box::new(plugin));
        self.plugins = Some(plugins);
    }

    pub fn get_render_base(&self) -> Arc<RenderBase> {
        self.render_base.clone()
    }

    pub fn simple_run<F>(mut self, mut func : F)
        where F : 'static + FnMut(&mut Game, winit::event::Event<'_, ()>, &mut winit::event_loop::ControlFlow) {
        let mut event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            func(&mut self, event, control_flow);
        });
    }

    fn resize_event(&mut self, new_size : PhysicalSize<u32>) {
        self.scene.world.insert_resource(new_size);
        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.window_resize(self, new_size);
        }
        self.plugins = Some(plugins);
        self.update_scene_scheldue();
    }

    fn camera_update(&mut self) {
        let speed = 0.3 / 5.0;
        if self.input.get_key_state(VirtualKeyCode::W) {
            self.scene.camera.pos += self.scene.camera.frw * speed;
        }
        if self.input.get_key_state(VirtualKeyCode::S) {
            self.scene.camera.pos -= self.scene.camera.frw * speed;
        }
        if self.input.get_key_state(VirtualKeyCode::D) {
            self.scene.camera.pos += self.scene.camera.get_right() * speed;
        }
        if self.input.get_key_state(VirtualKeyCode::A) {
            self.scene.camera.pos -= self.scene.camera.get_right() * speed;
        }
        if self.input.get_key_state(VirtualKeyCode::Space) {
            self.scene.camera.pos += self.scene.camera.up  * speed;
        }
        if self.input.get_key_state(VirtualKeyCode::LShift) {
            self.scene.camera.pos -= self.scene.camera.up * speed;
        }

        let mut encoder = self
            .render_base.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Update encoder"),
            });

        let camera_unifrom = self.scene.camera.build_uniform();
        let mut uniform = encase::UniformBuffer::new(vec![]);
        uniform.write(&camera_unifrom).unwrap();
        let inner = uniform.into_inner();

        let tmp_buffer = self.render_base.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &inner,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        encoder.copy_buffer_to_buffer(
            &tmp_buffer,
            0,
            &self.scene.camera_buffer,
            0,
            inner.len() as wgpu::BufferAddress);
        self.render_base.queue.submit(iter::once(encoder.finish()));
        self.render_base.device.poll(wgpu::Maintain::Wait);

    }

    fn update(&mut self) {
        let output = self.api.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());


        self.scene.world.insert_resource(RenderTarget {view, output});

        self.exec_commands();

        self.camera_update();

        self.scene.world.insert_resource(self.scene.camera.clone());
        self.scene.world.insert_resource(self.render_base.clone());
        self.scene.world.insert_resource(self.render_base.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        }));
        self.scene.world.get_resource_mut::<AssetServer>().unwrap().sync_tick();

        self.scene.scheduler.run(&mut self.scene.world);

        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.update(self);
        }
        self.plugins = Some(plugins);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let target = self.scene.world.remove_resource::<RenderTarget>().unwrap();
        let view = target.view;
        let output = target.output;
        self.render_view = Some(view);
        
        let mut plugins = self.plugins.take().unwrap();
        for p in plugins.render_plugin.iter_mut() {
            p.render(self);
        }
        self.plugins = Some(plugins);

        { //gui draw
            // let mut gui = self.scene.world.remove_resource::<Gui>().unwrap();
            // let gui_output = gui.end_frame(self.scene.world.get_resource());
            // let mut encoder = self.scene.world.remove_resource::<wgpu::CommandEncoder>().unwrap();

            // let scale_factor = self.scene.world.get_non_send_resource::<winit::window::Window>().unwrap().scale_factor();

            // gui.draw(gui_output,
            //               egui_wgpu_backend::ScreenDescriptor {
            //                   physical_width: self.api.config.width,
            //                   physical_height: self.api.config.height,
            //                   scale_factor: scale_factor as f32,
            //               },
            //               &mut encoder,
            //               &self.render_view.as_ref().unwrap());
            // self.scene.world.insert_resource(encoder);
            // self.scene.world.insert_resource(gui);

        }
        self.render_base.device.poll(wgpu::Maintain::Wait);
        self.render_base.queue.submit(Some(
            self.scene.world.remove_resource::<wgpu::CommandEncoder>().unwrap().finish()
        ));
        
        output.present();

        self.scene.world.get_resource_mut::<GpuProfiler>().unwrap().end_frame().unwrap();

        if let Some(profiling_data) =  self.scene.world.get_resource_mut::<GpuProfiler>().unwrap().process_finished_frame() {
            if self.input.get_key_state(winit::event::VirtualKeyCode::G) {
                wgpu_profiler::chrometrace::write_chrometrace(
                    std::path::Path::new("mytrace.json"), &profiling_data).unwrap();
            }
            // println!("Profile {}", profiling_data.len());
        }

        Ok(())
    }

    pub fn run(mut self){

        let mut event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            self.scene.world.get_resource_mut::<Gui>().unwrap().platform.handle_event(&event);
            let id = self.scene.world.get_resource::<winit::window::Window>().unwrap().id();
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == id => {
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
                            self.resize_event(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            self.resize_event(**new_inner_size);
                        }
                        WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {
                            self.input.process_event(input);
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == id => {
                    self.update();
                    match self.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if it's lost or outdated
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            let new_size = self.api.size.clone();
                            self.resize_event(new_size);
                        },
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                        Err(wgpu::SurfaceError::Timeout) => {},
                    }

                    if self.is_exit_state {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Event::RedrawEventsCleared => {
                    // RedrawRequested will only trigger once, unless we manually
                    // request it.
                    self.scene.world.get_resource_mut::<winit::window::Window>()
                        .unwrap().request_redraw();
                }
                _ => {}
            }
        });
    }

    fn create_window() -> (winit::window::Window, winit::event_loop::EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window =
            WindowBuilder::new().build(&event_loop).unwrap();
        window.set_title("Space sandbox");

        (window, event_loop)
    }

    pub fn add_schedule_plugin<T : SchedulePlugin + 'static>(&mut self, plugin : T) {
        let mut plugins = self.plugins.take().unwrap();
        plugins.scheldue_plugin.push(Box::new(plugin));
        self.plugins = Some(plugins);
    }

    pub fn update_scene_scheldue(&mut self) {
        let mut plugins = self.plugins.take().unwrap();

        let mut builder = Schedule::default();
        builder.add_stage(GlobalStageStep::RenderPrepare, SystemStage::parallel());
        builder.add_stage_after(GlobalStageStep::RenderPrepare,GlobalStageStep::Update, SystemStage::parallel());
        builder.add_stage_after(GlobalStageStep::Update,GlobalStageStep::PostUpdate, SystemStage::parallel());
        builder.add_stage_after(GlobalStageStep::RenderPrepare, GlobalStageStep::RenderStart, SystemStage::parallel());
        builder.add_stage_after(GlobalStageStep::RenderStart, GlobalStageStep::Render, SystemStage::single_threaded());
        builder.add_stage_after(GlobalStageStep::Render, GlobalStageStep::PostRender, SystemStage::single_threaded());
        builder.add_stage_after(GlobalStageStep::Update, GlobalStageStep::Gui, SystemStage::single_threaded());
        //push render prepare
        for plugin in &plugins.scheldue_plugin {
            plugin.add_system(self, &mut builder);
        }
        setup_gui(&mut self.scene.world, &mut builder);

        builder.add_system_to_stage(GlobalStageStep::RenderStart, poll_device);
        

        self.scene.scheduler = builder;
        self.plugins = Some(plugins);
    }
}


impl Default for Game {

    fn default() -> Self {

        let (window, event_loop) = Game::create_window();

        let api = ApiBase::new(&window);
        let render_base = api.render_base.clone();

        let gui = Gui::new(
            &render_base,
            api.config.format,
            wgpu::Extent3d {
                width : api.size.width,
                height : api.size.height,
                depth_or_array_layers : 1
            },
            window.scale_factor());
        let task_server = Arc::new(TaskServer::new());
        let assets = AssetServer::new(&render_base, &task_server);

        let camera = Camera::default();
        let camera_uniform = camera.build_uniform();

        let mut camera_cpu_buffer = UniformBuffer::new(vec![0u8;100]);
        camera_cpu_buffer.write(&camera_uniform);

        let camera_buffer = render_base.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Camera uniform buffer"),
            contents : &camera_cpu_buffer.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });


        let mut scene = GameScene {
            world : World::default(),
            scheduler : Schedule::default(),
            camera : Camera::default(),
            camera_buffer
        };

        scene.world.insert_resource(AssetServer::new(&render_base, &task_server));

        scene.world.insert_resource(
            GpuProfiler::new(
                4,
                render_base.queue.get_timestamp_period(),
                render_base.device.features()));

        scene.world.insert_resource(EguiContext {ctx : gui.platform.context()});
        scene.world.insert_resource(gui);
        scene.world.insert_non_send_resource(window);

        Self {
            event_loop : Some(event_loop),
            api,
            render_base,
            input : InputSystem::default(),
            plugins : Some(PluginBase::default()),
            render_view : None,
            task_server,
            scene,
            commands : vec![],
            is_exit_state : false
        }
    }
}