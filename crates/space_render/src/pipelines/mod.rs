use std::{fmt::Debug, sync::Arc};
use downcast_rs::{Downcast, impl_downcast};
use encase::*;

pub mod wgpu_gbuffer_fill;
pub mod wgpu_light_fill;
pub mod wgpu_texture_present;
pub mod wgpu_light_shadow;
pub mod wgpu_textures_transform;
pub mod wgpu_ssao;
pub mod wgpu_sreen_diffuse;
pub mod point_light_plugin;
pub mod wgpu_dir_light;

use space_assets::*;

use space_core::Camera;
use space_game::{Game, PluginName, PluginType, SchedulePlugin};
pub use wgpu_gbuffer_fill::*;
pub use wgpu_light_fill::*;
use wgpu_profiler::GpuProfiler;
pub use wgpu_texture_present::*;
pub use wgpu_light_shadow::*;
pub use wgpu_textures_transform::*;

use legion::*;
use legion::systems::Builder;
use crate::pipelines::wgpu_ssao::SSAOFrame;

use self::wgpu_sreen_diffuse::DepthTexture;

pub trait PipelineDesc : Downcast + Debug {
    fn get_shader_path(&self) -> AssetPath;
    fn set_shader_path(&mut self, path : AssetPath);
    fn clone_boxed(&self) -> Box<dyn PipelineDesc>;
}
impl_downcast!(PipelineDesc);

pub trait Pipeline {
    fn new_described(desc : Box<dyn PipelineDesc>, camera_buffer : &wgpu::Buffer) -> Self;
    fn get_desc(&self) -> Box<dyn PipelineDesc>;
}

#[derive(Default)]
pub struct DepthCalcUniform {
    pub cam_pos : [f32; 4]
}

impl TextureTransformUniform for DepthCalcUniform {
    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&self.cam_pos).to_vec()
    }
}

pub struct DepthPipeline {
    pipeline : TextureTransformPipeline
}

#[system]
fn fast_depth(
    #[resource] fill : &mut DepthPipeline,
    #[resource] gbuffer : &GFramebuffer,
    #[resource] encoder : &mut wgpu::CommandEncoder,
    #[resource] dst : &DepthTexture,
    #[resource] profiler : &mut GpuProfiler
) {

    profiler.begin_scope("Fast depth", encoder, &fill.pipeline.render.device);
    fill.pipeline.draw(encoder, &[&gbuffer.position], &[&dst.tex]);
    profiler.end_scope(encoder);

}

#[system]
fn fast_depth_update(
    #[resource] fill : &mut DepthPipeline,
    #[resource] camera : &Camera,
) {
    let depth_buffer = DepthCalcUniform {
        cam_pos : [camera.pos.x, camera.pos.y, camera.pos.z, 1.0]
    };
    fill.pipeline.update(Some(&depth_buffer));
}

pub struct FastDepthPlugin {
    
}

impl SchedulePlugin for FastDepthPlugin {
    fn get_name(&self) -> space_game::PluginName {
        space_game::PluginName::Text("FastDepth".into())
    }

    fn get_plugin_type(&self) -> space_game::PluginType {
        space_game::PluginType::Render
    }

    fn add_prepare_system(&self, game : &mut space_game::Game, builder : &mut legion::systems::Builder) {
        builder.add_system(fast_depth_update_system());
    }

    fn add_system(&self, game : &mut space_game::Game, builder : &mut legion::systems::Builder) {
        let depth_desc = TextureTransformDescriptor {
            render : game.render_base.clone(),
            format : wgpu::TextureFormat::R16Float,
            size : wgpu::Extent3d {
                width : game.api.size.width,
                height : game.api.size.height,
                depth_or_array_layers : 1
            },
            input_count : 1,
            output_count : 1,
            uniform : Some(Arc::new(DepthCalcUniform::default())),
            shader : include_str!("../../../../shaders/wgsl/depth_calc.wgsl").into(),
            blend : None,
            start_op : TextureTransformStart::Clear
        };

        let mut depth_calc = TextureTransformPipeline::new(
            &depth_desc
        );

        let mut common = depth_calc.spawn_framebuffer();
        let tex = common.dst.remove(0);

        let frame = DepthTexture {
            tex
        };

        builder.add_system(fast_depth_system());

        game.scene.resources.insert(DepthPipeline {
            pipeline : depth_calc
        });

        game.scene.resources.insert(frame);
    }
}

pub struct SSAOFiltered {
    pub tex : TextureBundle
}

pub struct SSAOFilter {
    pub pipeline : TextureTransformPipeline
}

#[system]
fn ssao_filter_impl(
    #[state] fill : &mut SSAOFilter,
    #[resource] dst : &SSAOFiltered,
    #[resource] ssao : &SSAOFrame,
    #[resource] depth : &DepthTexture,
    #[resource] encoder : &mut wgpu::CommandEncoder,
    #[resource] profiler : &mut GpuProfiler
) {
    profiler.begin_scope("SSAO smooth", encoder, &fill.pipeline.render.device);
    fill.pipeline.draw(encoder, &[&ssao.tex, &depth.tex], &[&dst.tex]);
    profiler.end_scope(encoder);
}

pub struct SSAOFilterSystem {

}

#[derive(ShaderType)]
pub struct SmoothUniform {
    pub size : nalgebra::Vector2<f32>
}

impl TextureTransformUniform for SmoothUniform {
    fn get_bytes(&self) -> Vec<u8> {
        let mut uniform = UniformBuffer::new(vec![]);
        uniform.write(&self);
        uniform.into_inner()
    }
}

impl SchedulePlugin for SSAOFilterSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("SSAO Filter".into())
    }

    fn get_plugin_type(&self) -> PluginType {
        PluginType::Render
    }

    fn add_system(&self, game: &mut Game, builder: &mut Builder) {

        let uniform = SmoothUniform {
            size : nalgebra::Vector2::new(game.api.size.width as f32, game.api.size.height as f32)
        };

        let pipeline = TextureTransformPipeline::new(&TextureTransformDescriptor {
            render: game.render_base.clone(),
            format: wgpu::TextureFormat::Rgba32Float,
            size: wgpu::Extent3d {
                width : game.api.size.width,
                height : game.api.size.height,
                depth_or_array_layers : 1
            },
            input_count: 2,
            output_count: 1,
            uniform: Some(Arc::new(uniform)),
            shader: include_str!("../../../../shaders/wgsl/smooth.wgsl").to_string(),
            blend: None,
            start_op: TextureTransformStart::Clear
        });

        let buffer = pipeline.spawn_framebuffer().dst.remove(0);

        game.scene.resources.insert(SSAOFiltered {tex : buffer});

        builder.add_system(ssao_filter_impl_system(SSAOFilter {pipeline}));
    }
}
