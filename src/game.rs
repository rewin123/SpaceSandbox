use std::collections::HashMap;
use std::sync::Arc;
use ash::vk;
use ash::vk::Extent2D;
use winit::event_loop::EventLoopWindowTarget;
use crate::{AllocatorSafe, BufferSafe, DeviceSafe, EguiWrapper, GraphicBase, Pools, RenderModel, TextureSafe, TextureServer};
use crate::asset_server::{AssetServer, BaseModels};
use crate::task_server::TaskServer;

pub struct PointLight {
    pub intensity : f32,
    pub position : [f32;3],
    pub color : [f32;3],
    pub instance : BufferSafe,
    pub shadow_map : Arc<TextureSafe>
}

impl PointLight {

    pub fn default(allocator : &Arc<AllocatorSafe>,
    device : &Arc<DeviceSafe>) -> Self {

        Self {
            intensity : 0.0,
            position : [0.0, 0.0 ,0.0],
            color : [1.0, 1.0, 1.0],
            instance : BufferSafe::new(
                allocator,
                PointLight::get_instance_stride() as u64,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu
            ).unwrap(),
            shadow_map : Arc::new(TextureSafe::new_depth_cubemap(
                allocator,
                device,
                Extent2D { width: 1024, height: 1024 },
                false))
        }
    }

    pub fn get_instance_stride() -> u32 {
        (1 + 3 + 3) * 4
    }

    pub fn get_instance_vertex_attribs() ->
         Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding : 3,
                location : 3,
                offset : 0,
                format: vk::Format::R32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding : 3,
                location : 4,
                offset : 4,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding : 3,
                location : 5,
                offset : 4 + 4 * 3,
                format: vk::Format::R32G32B32_SFLOAT
            },
        ]
    }

    pub fn fill_instanse(&mut self) {
        let mut data = vec![];
        data.push(self.intensity);
        data.extend(self.color);
        data.extend(self.position);
        self.instance.fill(&data);
    }
}


pub struct RenderServer {
    pub render_models : Vec<RenderModel>,
    pub point_lights : Vec<PointLight>
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
            render_server : RenderServer {
                render_models : vec![],
                point_lights : vec![]
            }
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