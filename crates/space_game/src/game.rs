use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::mem::swap;
use std::ops::{DerefMut, Deref};
use std::sync::{Arc, RwLock};
use atomic_refcell::AtomicRefMut;
use bevy::prelude::{App, CoreStage, info};
use egui::color::gamma_from_linear;
use wgpu::{Extent3d, ShaderStages, SurfaceTexture, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use space_assets::{SpaceAssetServer, Material, GMesh, LocationInstancing, LocationInstant};
use space_core::{Camera, RenderBase, TaskServer};
use crate::*;
use encase::*;
use wgpu::util::DeviceExt;
use space_core::bevy::asset::AssetPlugin;
use space_core::bevy::ecs::prelude::*;

fn update_instanced_loc(
        mut query : Query<(Entity, &mut LocationInstancing), Changed<LocationInstancing>>,
        render : Res<RenderApi>) {
    for  (entity, mut loc) in query.iter_mut() {
        info!("Update instancing {:?}", &entity);
        let mut cpu_buf = vec![LocationInstant::default(); loc.locs.len()];

        for idx in 0..loc.locs.len() {
            cpu_buf[idx] = loc.locs[idx].get_raw();
        }

        loc.buffer = Some(render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&cpu_buf),
            usage: wgpu::BufferUsages::VERTEX
        }));
    }
}

#[derive(Resource)]
pub struct WindowRes {
    pub window : winit::window::Window
}

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

#[derive(Resource)]
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
    pub app : App,
}


pub struct Game {
    pub api : ApiBase,
    event_loop : Option<winit::event_loop::EventLoop<()>>,
    pub render_base : Arc<RenderBase>,
    plugins : Option<PluginBase>,
    pub render_view : Option<TextureView>,
    pub task_server : Arc<TaskServer>,
    pub scene : GameScene,
    pub commands : Vec<GameCommands>,
    pub is_exit_state : bool
}

fn poll_device( render_base : Res<RenderApi>) {
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

    pub fn get_assets(&mut self) -> Mut<SpaceAssetServer> {
        self.scene.app.world.get_resource_mut::<SpaceAssetServer>().unwrap()
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
        self.scene.app.insert_resource(ScreenSize {size : new_size.clone(), format : self.api.config.format.clone()});
        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.window_resize(self, new_size);
        }
        self.plugins = Some(plugins);
        // self.update_scene_scheldue();
    }

    fn camera_update(&mut self) {
        let mut encoder = self
            .render_base.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Update encoder"),
            });


        let camera_unifrom = self.scene.app.world.get_resource::<Camera>().unwrap().build_uniform();
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
            &self.scene.app.world.get_resource::<CameraBuffer>().unwrap().buffer,
            0,
            inner.len() as wgpu::BufferAddress);
        self.render_base.queue.submit(iter::once(encoder.finish()));
        self.render_base.device.poll(wgpu::Maintain::Wait);

    }

    fn update(&mut self) {
        let output;
        if let Ok(val) = self.api.surface.get_current_texture() {
            output = val;
        } else {
            return;
        }
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());


        self.scene.app.insert_resource(RenderTarget {view, output});

        self.exec_commands();

        self.camera_update();

        self.scene.app.insert_resource(RenderApi {base : self.render_base.clone()});
        self.scene.app.insert_resource( RenderCommands{ encoder : self.render_base.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        })});
        self.scene.app.world.get_resource_mut::<SpaceAssetServer>().unwrap().sync_tick();

        self.scene.app.update();

        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.update(self);
        }
        self.plugins = Some(plugins);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let target;
        if let Some(val) = self.scene.app.world.remove_resource::<RenderTarget>() {
            target = val;
        } else {
            return Ok(());
        }
        let view = target.view;
        let output = target.output;
        self.render_view = Some(view);
        
        let mut plugins = self.plugins.take().unwrap();
        for p in plugins.render_plugin.iter_mut() {
            p.render(self);
        }
        self.plugins = Some(plugins);

        self.render_base.device.poll(wgpu::Maintain::Wait);
        self.render_base.queue.submit(Some(
            self.scene.app.world.remove_resource::<RenderCommands>().unwrap().encoder.finish()
        ));
        
        output.present();

        Ok(())
    }

    pub fn run(mut self){

        let mut event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            if let Some(mut gui) = self.scene.app.world.get_resource_mut::<Gui>() {
                gui.platform.handle_event(&event);
            }
            let id = self.scene.app.world.get_non_send_resource::<winit::window::Window>().unwrap().id();
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
                            self.scene.app.world.get_resource_mut::<InputSystem>()
                                .unwrap().process_event(input);
                        }
                        WindowEvent::MouseInput { device_id, state, button, modifiers } => {
                            self.scene.app.world.get_resource_mut::<InputSystem>()
                                .unwrap().process_mouse_event(button, state);
                        }
                        WindowEvent::CursorMoved { device_id, position, modifiers } => {
                            self.scene.app.world.get_resource_mut::<InputSystem>()
                                .unwrap().process_cursor_move(position.clone());
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
                    self.scene.app.world.get_non_send_resource_mut::<winit::window::Window>()
                        .unwrap().request_redraw();
                }
                _ => {}
            }
        });
    }

    fn create_window() -> (winit::window::Window, winit::event_loop::EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window =
            WindowBuilder::new()
            .build(&event_loop).unwrap();
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

        println!("Update scene schedule");

        self.scene.app.schedule = Schedule::default();
        self.scene.app.add_default_stages();
        self.scene.app.add_plugin(bevy::log::LogPlugin::default());
        self.scene.app.add_plugins(bevy::MinimalPlugins);
        self.scene.app.add_plugin(bevy::diagnostic::DiagnosticsPlugin::default());
        self.scene.app.add_plugin(AssetPlugin::default());

        self.scene.app.add_asset::<Material>();
        self.scene.app.add_asset::<GMesh>();

        self.scene.app.add_stage_after(CoreStage::PreUpdate, GlobalStageStep::PreRender, SystemStage::parallel());
        self.scene.app.add_stage_after(GlobalStageStep::PreRender, GlobalStageStep::Render, SystemStage::single_threaded());
        self.scene.app.add_stage_after(GlobalStageStep::Render, GlobalStageStep::PostRender, SystemStage::single_threaded());
        self.scene.app.add_stage_after(GlobalStageStep::PostRender, GlobalStageStep::Gui, SystemStage::single_threaded());
        self.scene.app.add_system_to_stage(GlobalStageStep::Render, poll_device);
        self.scene.app.add_state(SceneType::MainMenu);
        //push render prepare
        for plugin in &plugins.scheldue_plugin {
            plugin.add_system(&mut self.scene.app);
        }

        self.scene.app.add_system_to_stage(CoreStage::PreUpdate, update_instanced_loc);

        setup_gui(&mut self.scene.app);
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
        let assets = SpaceAssetServer::new(&render_base, &task_server);

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
            app : App::default(),
        };

        scene.app.insert_resource(SpaceAssetServer::new(&render_base, &task_server));
        scene.app.insert_resource(Camera::default());

        // scene.app.insert_resource(
        //     GpuProfiler::new(
        //         4,
        //         render_base.queue.get_timestamp_period(),
        //         render_base.device.features()));

        scene.app.insert_resource(EguiContext {ctx : gui.platform.context()});
        scene.app.insert_resource(gui);
        scene.app.insert_non_send_resource(window);

        scene.app.insert_resource(RenderApi {base : render_base.clone()});
        scene.app.insert_resource(ScreenSize {size : api.size.clone(), format : api.config.format});

        scene.app.insert_resource(CameraBuffer {buffer : camera_buffer});

        scene.app.insert_resource(InputSystem::default());

        Self {
            event_loop : Some(event_loop),
            api,
            render_base,
            plugins : Some(PluginBase::default()),
            render_view : None,
            task_server,
            scene,
            commands : vec![],
            is_exit_state : false
        }
    }
}