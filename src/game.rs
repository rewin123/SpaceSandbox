use std::collections::HashMap;
use std::sync::Arc;
use winit::event_loop::EventLoopWindowTarget;
use crate::{EguiWrapper, GraphicBase, Pools, RenderModel, TextureServer};
use crate::asset_server::AssetServer;
use crate::task_server::TaskServer;

pub struct RenderServer {
    pub render_models : Vec<RenderModel>
}

pub struct Game {
    pub world : specs::World,
    pub task_server : Arc<TaskServer>,
    pub gb : GraphicBase,
    pub pools : Arc<Pools>,
    pub gui : EguiWrapper,
    pub event_loop : Option<winit::event_loop::EventLoop<()>>,
    pub render_server : RenderServer
}


impl Default for Game {
    fn default() -> Self {
        let eventloop = winit::event_loop::EventLoop::new();
        let window = winit::window::Window::new(&eventloop).unwrap();

        let mut graphic_base = GraphicBase::init(window);

        let pools = Pools::init(
            &graphic_base.device,
            &graphic_base.queue_families
        ).unwrap();

        let mut gui = EguiWrapper::new(
            &graphic_base
        );

        let mut task_server = Arc::new(TaskServer::new());

        Self {
            world: Default::default(),
            task_server: task_server,
            gb: graphic_base,
            pools,
            gui,
            event_loop: Some(eventloop),
            render_server : RenderServer {render_models : vec![]}
        }
    }
}

impl Game {
    pub fn simple_loop<F>(mut self, mut f : F)
        where F: 'static + FnMut(
            &mut Game,
            winit::event::Event<'_, ()>,
            &EventLoopWindowTarget<()>,
            &mut winit::event_loop::ControlFlow) {

        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, target, controlflow| {
            self.gui.integration.handle_event(&event);

            f(&mut self, event, target, controlflow);
        });
    }
}