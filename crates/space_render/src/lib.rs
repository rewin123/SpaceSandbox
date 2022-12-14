pub mod pipelines;
pub mod light;
pub mod hdri;
pub mod ui;

use bevy::prelude::Component;
use nalgebra as na;
use encase::*;
use space_game::*;
use space_game::plugins::FpsCounterSystem;
use crate::pipelines::StateSystem;

#[derive(Component)]
pub struct AutoInstancing {}

pub fn add_game_render_plugins(game : &mut Game) {
    // let mut state = pollster::block_on(crate::pipelines::State::new(game));
    // game.add_schedule_plugin(StateSystem{});
    // game.add_schedule_plugin(space_game::plugins::LocUpdateSystem {});
    // game.add_schedule_plugin(crate::pipelines::GBufferPlugin {});
    // game.add_schedule_plugin(crate::pipelines::point_light_plugin::PointLightPlugin {});
    // game.add_schedule_plugin(crate::pipelines::FastDepthPlugin {});
    // game.add_schedule_plugin(crate::pipelines::wgpu_sreen_diffuse::SSDiffuseSystem {});
    // game.add_schedule_plugin(crate::pipelines::SSAOFilterSystem {});
    // game.add_schedule_plugin(crate::pipelines::wgpu_dir_light::DirLightSystem {});
    // game.add_schedule_plugin(crate::hdri::HDRISystem {path : "res/hdri/space/outer-space-background.jpg".into()});
    // game.add_schedule_plugin(FpsCounterSystem {});
}