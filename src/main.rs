use std::iter;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use bytemuck::{Zeroable, Pod};
use egui::epaint::ahash::HashMap;
use egui_gizmo::GizmoMode;
use egui_wgpu_backend::ScreenDescriptor;
use space_render::pipelines::wgpu_sreen_diffuse::{SSDiffuse, DepthTexture, SSDiffuseSystem};
use space_shaders::*;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use SpaceSandbox::{init_logger};
use encase::{ShaderType, UniformBuffer};
use image::gif::Encoder;
use space_assets::*;
use wgpu_profiler::*;

use nalgebra as na;
use nalgebra::Matrix4;
use wgpu::{BlendFactor, MaintainBase};
use space_core::{RenderBase, TaskServer};
use space_render::{add_game_render_plugins, pipelines::*};
use space_render::light::*;
use space_render::pipelines::wgpu_ssao::{SSAO, SSAOFrame};

use legion::*;
use space_assets::wavefront::wgpu_load_gray_obj;
use space_game::{Game, RenderPlugin};
use space_game::plugins::LocUpdateSystem;
use space_render::hdri::HDRISystem;
use space_render::pipelines::point_light_plugin::PointLightPlugin;
use space_render::pipelines::wgpu_dir_light::{DirLight, DirLightSystem};

use space_shaders::*;
use space_render::pipelines::State;

async fn run() {
    init_logger();
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();
    let mut game = Game::default();
    add_game_render_plugins(&mut game).await;
    game.update_scene_scheldue();
    game.run();
}

fn main() {
    pollster::block_on(run());
}