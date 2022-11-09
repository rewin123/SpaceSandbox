use std::cell::RefCell;
use std::collections::HashMap;
use std::iter;
use std::mem::swap;
use std::ops::DerefMut;
use std::sync::Arc;
use atomic_refcell::AtomicRefMut;
use egui::color::gamma_from_linear;
use legion::systems::Builder;
use wgpu::{Extent3d, SurfaceTexture, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use space_assets::{AssetServer, Material};
use space_core::{Camera, RenderBase, TaskServer};
use crate::{ApiBase, GameCommands, Gui, GuiPlugin, InputSystem, PluginType, RenderPlugin, SchedulePlugin};
use encase::*;
use wgpu::util::DeviceExt;
use legion::*;
use wgpu_profiler::*;

#[derive(Default)]
pub struct PluginBase {
    gui_plugins : Vec<Box<dyn GuiPlugin>>,
    render_plugin : Vec<Box<dyn RenderPlugin>>,
    scheldue_plugin : Vec<Box<dyn SchedulePlugin>>
}

pub struct GameScene {
    pub world : World,
    pub resources : Resources,
    pub scheduler : Schedule,
    pub camera : Camera,
    pub camera_buffer : wgpu::Buffer,
}


pub struct Game {
    pub window : winit::window::Window,
    pub api : ApiBase,
    event_loop : Option<winit::event_loop::EventLoop<()>>,
    pub render_base : Arc<RenderBase>,
    pub input : InputSystem,
    pub gui : Gui,
    plugins : Option<PluginBase>,
    pub render_view : Option<TextureView>,
    pub task_server : Arc<TaskServer>,
    pub scene : GameScene,
    pub commands : Vec<GameCommands>,
    pub is_exit_state : bool
}

#[system]
fn poll_device(#[resource] render_base : &Arc<RenderBase>) {
    render_base.device.poll(wgpu::Maintain::Wait);
}


impl Game {

    pub fn exec_commands(&mut self) {
        let mut cmds = vec![];
        swap(&mut cmds, &mut self.commands);

        for cmd in &cmds {
            match cmd {
                GameCommands::Exit => {
                    self.is_exit_state = true;
                }
            }
        }
    }

    pub fn add_gui_plugin<T : GuiPlugin + 'static>(&mut self, plugin : T) {
        let mut plugins = self.plugins.take().unwrap();
        plugins.gui_plugins.push(Box::new(plugin));
        self.plugins = Some(plugins);
    }

    pub fn get_default_material(&self) -> Material {
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

    pub fn get_assets(&self) -> AtomicRefMut<AssetServer> {
        self.scene.resources.get_mut::<AssetServer>().unwrap()
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
        self.scene.resources.insert(new_size);
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
        self.exec_commands();

        self.camera_update();

        self.scene.resources.insert(self.scene.camera.clone());
        self.scene.resources.insert(self.render_base.clone());
        self.scene.resources.insert(self.render_base.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        }));
        self.scene.resources.get_mut::<AssetServer>().unwrap().sync_tick();
        self.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope(
            "Global scopre", 
            self.scene.resources.get_mut::<wgpu::CommandEncoder>().unwrap().deref_mut(),
            &self.render_base.device);
        self.scene.scheduler.execute(&mut self.scene.world, &mut self.scene.resources);
        self.scene.resources.get_mut::<GpuProfiler>().unwrap().end_scope(
            self.scene.resources.get_mut::<wgpu::CommandEncoder>().unwrap().deref_mut());

        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.update(self);
        }
        self.plugins = Some(plugins);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.api.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_view = Some(view);

        let mut plugins = self.plugins.take().unwrap();
        {
            for plugin in &mut plugins.render_plugin {
                plugin.render(self);
            }

            self.gui.begin_frame();
            egui::TopBottomPanel::top("top_panel").show(
                &self.gui.platform.context(), |ui| {
                    ui.horizontal(|ui| {
                        for plugin in &mut plugins.gui_plugins {
                            let cmds =
                                plugin.shot_top_panel(self, ui);
                            self.commands.extend(cmds);
                        }
                        for plugin in &mut plugins.render_plugin {
                            plugin.show_top_panel(self, ui);
                        }
                    });
                });

            for plugin in &mut plugins.gui_plugins {
                let cmds =
                    plugin.show_ui(self, self.gui.platform.context());
                self.commands.extend(cmds);
            }
            for plugin in &mut plugins.render_plugin {
                plugin.show_ui(self, self.gui.platform.context());
            }
        }
        self.plugins = Some(plugins);

        { //gui draw
            let gui_output = self.gui.end_frame(Some(&self.window));
            let mut encoder_ref = self.scene.resources.get_mut::<wgpu::CommandEncoder>().unwrap();
            let encoder = encoder_ref.deref_mut();
            self.gui.draw(gui_output,
                          egui_wgpu_backend::ScreenDescriptor {
                              physical_width: self.api.config.width,
                              physical_height: self.api.config.height,
                              scale_factor: self.window.scale_factor() as f32,
                          },
                          encoder,
                          &self.render_view.as_ref().unwrap());
        }

        self.scene.resources.get_mut::<GpuProfiler>().unwrap().resolve_queries(
            self.scene.resources.get_mut::<wgpu::CommandEncoder>().unwrap().deref_mut()
        );

        self.render_base.queue.submit(Some(
            self.scene.resources.remove::<wgpu::CommandEncoder>().unwrap().finish()
        ));
        
        output.present();

        self.scene.resources.get_mut::<GpuProfiler>().unwrap().end_frame().unwrap();

        if let Some(profiling_data) =  self.scene.resources.get_mut::<GpuProfiler>().unwrap().process_finished_frame() {
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
            self.gui.platform.handle_event(&event);
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => {
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
                Event::RedrawRequested(window_id) if window_id == self.window.id() => {
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
                    self.window.request_redraw();
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

        let mut builder = Schedule::builder();
        //push render prepare
        for plugin in &plugins.scheldue_plugin {
            if plugin.get_plugin_type() == PluginType::RenderPrepare {
                plugin.add_system(self, &mut builder);
            } else {
                plugin.add_prepare_system(self, &mut builder);
            }
        }
        builder.flush();
        builder.add_system(poll_device_system());
        builder.flush();
        for plugin in &plugins.scheldue_plugin {
            if plugin.get_plugin_type() != PluginType::RenderPrepare {
                plugin.add_system(self, &mut builder);
            }
        }
        builder.flush();
        self.scene.scheduler = builder.build();
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
            resources : Resources::default(),
            scheduler : Schedule::builder().build(),
            camera : Camera::default(),
            camera_buffer
        };

        scene.resources.insert(AssetServer::new(&render_base, &task_server));

        scene.resources.insert(
            GpuProfiler::new(
                4,
                render_base.queue.get_timestamp_period(),
                render_base.device.features()));

        Self {
            window,
            event_loop : Some(event_loop),
            api,
            render_base,
            input : InputSystem::default(),
            gui,
            plugins : Some(PluginBase::default()),
            render_view : None,
            task_server,
            scene,
            commands : vec![],
            is_exit_state : false
        }
    }
}