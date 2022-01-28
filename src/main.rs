use std::{borrow::Cow, future::Future};
use winit::{
    event::{Event, WindowEvent, StartCause},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};
use engine::{resource::*, mesh::*};
use wgpu::{util::DeviceExt, TextureView};
use bytemuck::{Pod, Zeroable};
use std::sync::mpsc;
use std::thread;
use engine::loop_game::{LoopGame, LoopGameEvent};
use engine::camera::*;

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
    let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
    let mx_view = cgmath::Matrix4::look_at_rh(
        cgmath::Point3::new(1.5f32, -5.0, 3.0),
        cgmath::Point3::new(0f32, 0.0, 0.0),
        cgmath::Vector3::unit_z(),
    );
    let mx_correction = OPENGL_TO_WGPU_MATRIX;
    mx_correction * mx_projection * mx_view
}


struct ModelViewGame {
    engine : engine::Engine,
    render : engine::render::DepthRender,
    angle : f32,
    camera : Camera,
    gpu_mesh : engine::mesh::GPUMesh
}


impl ModelViewGame {
    pub async fn new(window : &Window) -> Self {

        let engine = engine::Engine::from_window(window, &String::from("./res")).await;

        let depth_view = engine.create_screen_depth_texture();

        let mut camera = Camera {
            uniform : CameraUniform {
                pos : Vec4::default(),
                frw : Vec4::default(),
                up : Vec4::default()
            }
        };
    
        camera.uniform.pos.w = 1.0;
        camera.uniform.frw.w = 1.0;
        camera.uniform.up.w = 1.0;
    
        camera.uniform.pos.x = -3.0;
        camera.uniform.frw.x = 1.0;
        camera.uniform.up.z = 1.0;

        let gpu_mesh = engine.load_gpu_mesh(&String::from("tomokitty")).unwrap();

        let render = engine::render::DepthRender::from_engine(&engine);

        Self {
            engine,
            angle : 0.0, 
            camera,
            gpu_mesh,
            render
        }
    }
}

impl engine::loop_game::LoopGame for ModelViewGame {
    fn init(&mut self, base : &engine::loop_game::LoopGameBase) {
        
    }

    fn logick_loop(&mut self) {
        
    }

    fn draw_loop(&mut self) {
        self.angle += 0.001;
        let sval = self.angle.sin();
        let cval = self.angle.cos();
        let distance = 5.0;

        self.camera.uniform.pos = Vec4 {
            x : sval * distance,
            y : cval * distance,
            z : 0.0,
            w : 1.0
        };
        self.camera.uniform.frw = Vec4 {
            x : -sval,
            y : -cval,
            z : 0.0,
            w : 1.0
        };

        self.render.raw_draw(&self.gpu_mesh, &self.camera, &self.engine.gpu);
    }

    fn resize_event(&mut self, size : &winit::dpi::PhysicalSize<u32>) {
        
        self.engine.gpu.resize(size.width, size.height);
        self.render.depth_view = engine::gpu::GPU::create_depth_texture(&self.engine.gpu.surface_config, &self.engine.gpu.device);
    }
}

#[tokio::main]
async fn main() {
    let base_loop = engine::loop_game::LoopGameBase::default();
    let mut my_game = ModelViewGame::new(&base_loop.window).await;

    let (tx, rx) = mpsc::channel();
    let handler = thread::spawn(move || {
        let mut game_running = true;
        while game_running {
            match rx.try_recv() {
                Ok(val) => {

                    match val {
                        LoopGameEvent::Redraw => {
                            my_game.draw_loop();

                        }
                        LoopGameEvent::Exit => {
                            game_running = false;
                        }
                        LoopGameEvent::Resize(size) => {

                        }
                        LoopGameEvent::None => {

                        }
                    }
                }
                Err(err) => {

                }
            }
        }
    });



    base_loop.run(tx);

    // Ok(())
}