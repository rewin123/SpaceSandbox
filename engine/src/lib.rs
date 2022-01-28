
pub mod resource;
pub mod wavefront;
pub mod mesh;
pub mod gpu;
pub mod loop_game;
pub mod camera;


use crate::{resource::*};
use crate::gpu::GPU;
use winit::window::Window;

pub struct Engine {
    gpu : GPU, 
    file_res_system : FileResourceEngine
}

impl Engine {
    pub async fn from_window(window : &Window, res_path : &String) -> Self {
        let gpu = gpu::GPU::from_window(window).await;

        let mut file_res_system = FileResourceEngine::default();
        file_res_system.init(res_path);

        Self {
            gpu,
            file_res_system
        }
    }
}