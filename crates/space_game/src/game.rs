use std::collections::HashMap;
use std::iter;
use std::sync::Arc;
use legion::{Resources, Schedule, World};
use legion::systems::Builder;
use wgpu::{SurfaceTexture, TextureView};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use space_assets::AssetServer;
use space_core::{RenderBase, TaskServer};
use crate::{ApiBase, Gui, GuiPlugin, InputSystem, PluginType, RenderPlugin, SchedulePlugin};

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
    pub assets : AssetServer,
    pub scene : GameScene
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



        let scene = GameScene {
            world : World::default(),
            resources : Resources::default(),
            scheduler : Schedule::builder().build()
        };

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
            assets,
            scene
        }
    }
}

impl Game {

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
        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.window_resize(self, new_size);
        }
        self.plugins = Some(plugins);
    }

    fn update(&mut self) {
        self.scene.scheduler.execute(&mut self.scene.world, &mut self.scene.resources);

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

        let mut encoder = self
            .render_base.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let mut plugins = self.plugins.take().unwrap();
        for plugin in &mut plugins.render_plugin {
            plugin.render(self, &mut encoder);
        }
        self.plugins = Some(plugins);

        self.render_base.queue.submit(iter::once(encoder.finish()));
        output.present();

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
        let mut plugins = self.plugins.as_ref().unwrap();

        let mut builder = Schedule::builder();
        //push render prepare
        for plugin in &plugins.scheldue_plugin {
            if plugin.get_plugin_type() == PluginType::RenderPrepare {
                plugin.add_system(&self, &mut builder);
            }
        }
        builder.flush();
        for plugin in &plugins.scheldue_plugin {
            if plugin.get_plugin_type() != PluginType::RenderPrepare {
                plugin.add_system(&self, &mut builder);
            }
        }
        builder.flush();
        self.scene.scheduler = builder.build();
    }
}
