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
use space_render::{pipelines::*};
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

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new().await;

    let mut game = state.game.take().unwrap();
    game.add_render_plugin(state);
    game.add_schedule_plugin(LocUpdateSystem{});
    game.add_schedule_plugin(GBufferPlugin{});
    game.add_schedule_plugin(PointLightPlugin{});
    game.add_schedule_plugin(FastDepthPlugin{});
    game.add_schedule_plugin(SSDiffuseSystem{});
    game.add_schedule_plugin(SSAOFilterSystem{});
    game.add_schedule_plugin(DirLightSystem{});
    game.add_schedule_plugin(HDRISystem{path : "res/hdri/space/outer-space-background.jpg".into()});
    game.update_scene_scheldue();

    {
        let mut assets = game.scene.resources.get_mut::<AssetServer>().unwrap();
        assets.wgpu_gltf_load(
            &game.render_base.device,
            "res/bobik/bobik.gltf".into(),
            &mut game.scene.world);
    }
    let mut light =
        PointLight::new(&game.render_base, [0.0, 3.0, 0.0].into(), false);
    light.intensity = 20.0;
    game.scene.world.push((light,));
    let mut dir_light = DirLight::default(&game.render_base);
    game.scene.world.push((dir_light,));

    game.run();
}


fn main() {
    pollster::block_on(run());
}