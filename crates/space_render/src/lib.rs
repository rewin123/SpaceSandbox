pub mod pipelines;
pub mod light;
pub mod hdri;
pub mod ui;

use nalgebra as na;
use encase::*;
use legion::systems::Builder;
use space_game::{Game, PluginName, PluginType, SchedulePlugin};


pub fn add_game_render_plugins(game : &mut Game) {
    let mut state = pollster::block_on(crate::pipelines::State::new(game));
    game.add_render_plugin(state);
    game.add_schedule_plugin(space_game::plugins::LocUpdateSystem {});
    game.add_schedule_plugin(crate::pipelines::GBufferPlugin {});
    game.add_schedule_plugin(crate::pipelines::point_light_plugin::PointLightPlugin {});
    game.add_schedule_plugin(crate::pipelines::FastDepthPlugin {});
    game.add_schedule_plugin(crate::pipelines::wgpu_sreen_diffuse::SSDiffuseSystem {});
    game.add_schedule_plugin(crate::pipelines::SSAOFilterSystem {});
    game.add_schedule_plugin(crate::pipelines::wgpu_dir_light::DirLightSystem {});
    game.add_schedule_plugin(crate::hdri::HDRISystem {path : "res/hdri/space/outer-space-background.jpg".into()});
}