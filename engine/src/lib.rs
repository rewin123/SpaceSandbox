
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
use winit::window::Window;
use legion::*;

pub struct Engine {
    pub gpu : GPU, 
    pub file_res_system : FileResourceEngine,
    pub world : World
}

impl Engine {
    pub async fn from_window(window : &Window, res_path : &String) -> Self {
        let gpu = gpu::GPU::from_window(window).await;

        let mut file_res_system = FileResourceEngine::default();
        file_res_system.init(res_path);

        Self {
            gpu,
            file_res_system,
            world : World::default()
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