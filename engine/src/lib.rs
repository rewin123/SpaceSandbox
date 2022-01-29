
pub mod resource;
pub mod wavefront;
pub mod mesh;
pub mod gpu;
pub mod loop_game;
pub mod camera;
pub mod render;
pub mod gui;

use crate::{resource::*, mesh::*};
use crate::gpu::GPU;
use legion::*;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window, dpi::PhysicalSize,
};

pub struct GameProgram<T> {
    pub engine : Engine,
    pub game : T
}

impl<T> GameProgram<T> where T : LoopGame + 'static {
    pub async fn run(path : &String) {
        let event_loop = EventLoop::new();
        let window = winit::window::Window::new(&event_loop).unwrap();

        let mut engine = Engine::from_window(&window, path).await;
        let mut game = T::from_engine(&window, &mut engine);

        event_loop.run(move  |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            engine.gui_render.platform.handle_event(&event);

            window.request_redraw();

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    game.resize_event(&size, &mut engine);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::RedrawRequested(id) => {
                    game.draw_loop(&mut engine);
                }
                _ => {}
            }
        });
    }
}

pub trait LoopGame {
    
    fn from_engine(window : &Window, engine : &mut Engine) -> Self;
    fn logick_loop(&mut self);
    fn draw_loop(&mut self, engine : &mut Engine);
    fn resize_event(&mut self, size : &PhysicalSize<u32>, engine : &mut Engine);
}

pub struct Engine {
    pub gpu : GPU, 
    pub file_res_system : FileResourceEngine,
    pub world : World,
    pub gui_render : gui::GUIRender
}

impl Engine {

    pub async fn from_window(window : &Window, res_path : &String) -> Self {
        let gpu = gpu::GPU::from_window(&window).await;

        let mut file_res_system = FileResourceEngine::default();
        file_res_system.init(res_path);

        let gui_render = gui::GUIRender::new(&window, &gpu);

        Self {
            gpu,
            file_res_system,
            world : World::default(),
            gui_render
        }
    }

    pub fn create_screen_depth_texture(&self) -> wgpu::TextureView {
        GPU::create_depth_texture(&self.gpu.surface_config, &self.gpu.device)
    }

    pub fn load_shader_module(&self, path : &String) -> wgpu::ShaderModule {
        self.gpu.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: self.file_res_system.get_wgsl_shader(path).unwrap()
        })
    }

    pub fn load_gpu_mesh(&self, path : &String) -> Result<GPUMesh, String> {
        match self.file_res_system.get_data_string(path) {
            Some(data) => {
                match crate::wavefront::SimpleWavefrontParser::from_str(&data) {
                    Err(err) => {
                        Err(err)
                    }
                    Ok(mesh) => {
                        Ok(GPUMesh::from(&self.gpu, &mesh))
                    }
                }
            }
            None => {
               Err(String::from("Cannot load mesh file"))
            }
        }
    }

}