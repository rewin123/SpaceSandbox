use std::{fmt::Debug, sync::Arc};
use std::ops::DerefMut;
use bevy::prelude::Assets;
use downcast_rs::{Downcast, impl_downcast};
use egui::{Context, Ui};
use encase::*;
use nalgebra as na;

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

use space_core::app::App;
use space_core::{Camera, RenderBase};
use space_game::*;
pub use wgpu_gbuffer_fill::*;
pub use wgpu_light_fill::*;
pub use wgpu_texture_present::*;
pub use wgpu_light_shadow::*;
pub use wgpu_textures_transform::*;

use crate::light::{AmbientLightUniform, PointLight};
use crate::pipelines::wgpu_ssao::SSAOFrame;

use self::wgpu_sreen_diffuse::DepthTexture;

use space_core::ecs::*;

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

#[derive(Resource)]
pub struct DepthPipeline {
    pipeline : TextureTransformPipeline
}

fn fast_depth(
    mut fill : ResMut<DepthPipeline>,
    gbuffer : Res<GFramebuffer>,
    mut encoder : ResMut<RenderCommands>,
    dst : Res<DepthTexture>
) {

    // profiler.begin_scope("Fast depth", encoder, &fill.pipeline.render.device);
    fill.pipeline.draw(encoder.as_mut(), &[&gbuffer.position], &[&dst.tex]);
    // profiler.end_scope(encoder);

}

fn fast_depth_update(
    mut fill : ResMut<DepthPipeline>,
    camera : Res<Camera>,
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

    fn add_system(&self, app : &mut App) {

        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();

        let depth_desc = TextureTransformDescriptor {
            render : app.world.get_resource::<RenderApi>().unwrap().base.clone(),
            format : wgpu::TextureFormat::R16Float,
            size : wgpu::Extent3d {
                width : size.width,
                height : size.height,
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

        app.add_system_to_stage(GlobalStageStep::Render, fast_depth);
        app.add_system_to_stage(GlobalStageStep::PreRender, fast_depth_update);

        app.insert_resource(DepthPipeline {pipeline : depth_calc});
        app.insert_resource(frame);
    }
}

#[derive(Resource)]
pub struct SSAOFiltered {
    pub tex : TextureBundle
}

#[derive(Resource)]
pub struct SSAOFilter {
    pub pipeline : TextureTransformPipeline
}

fn ssao_filter_impl(
    mut fill : ResMut<SSAOFilter>,
    dst : Res<SSAOFiltered>,
    ssao : Res<SSAOFrame>,
    depth : Res<DepthTexture>,
    mut encoder : ResMut<RenderCommands>
) {
    fill.pipeline.draw(encoder.as_mut(), &[&ssao.tex, &depth.tex], &[&dst.tex]);
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

    fn add_system(&self, app: &mut App) {

        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();
        let render = app.world.get_resource::<RenderApi>().unwrap().base.clone();

        let uniform = SmoothUniform {
            size : nalgebra::Vector2::new(size.width as f32, size.height as f32)
        };

        let pipeline = TextureTransformPipeline::new(&TextureTransformDescriptor {
            render: render.clone(),
            format: wgpu::TextureFormat::Rgba32Float,
            size: wgpu::Extent3d {
                width : size.width,
                height : size.height,
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

        app.insert_resource(SSAOFiltered {tex : buffer});
        app.insert_resource(SSAOFilter {pipeline});

        app.add_system_to_stage(GlobalStageStep::Render, ssao_filter_impl);
    }
}


#[derive(Debug, PartialEq)]
enum DrawState {
    Full,
    DirectLight,
    AmbientOcclusion,
    AmbientOcclusionSmooth,
    Depth
}

#[derive(Resource)]
pub struct State {
    render : Arc<RenderBase>,

    gamma_correction : TextureTransformPipeline,
    present : TexturePresent,
    gamma_buffer : CommonFramebuffer,

    draw_state : DrawState,
    ambient_light : crate::light::AmbientLight,
    ambient_light_pipeline : TextureTransformPipeline
}


impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(app : &mut App) -> Self {
        let render = app.world.get_resource::<RenderApi>().unwrap().base.clone();
        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();
        let screen_format = app.world.get_resource::<ScreenSize>().unwrap().format.clone();

        let extent = wgpu::Extent3d {
            width : size.width,
            height : size.height,
            depth_or_array_layers : 1
        };


        let framebuffer = GBufferFill::spawn_framebuffer(
            &render.device,
            extent);

        let present = TexturePresent::new(
            &render,
            screen_format,
            wgpu::Extent3d {
                width : size.width,
                height : size.height,
                depth_or_array_layers : 1
            });
        let point_light_shadow = PointLightShadowPipeline::new(&render);

        let light_pipeline = PointLightPipeline::new(
            &render, 
            extent,
            app);
        let light_buffer = light_pipeline.spawn_framebuffer(&render.device, extent);

        let gamma_desc = TextureTransformDescriptor {
            render : render.clone(),
            format: wgpu::TextureFormat::Rgba32Float,
            size: extent,
            input_count: 1,
            output_count: 1,
            uniform: None,
            shader: include_str!("../../../../shaders/wgsl/gamma_correction.wgsl").into(),
            blend : None,
            start_op : TextureTransformStart::Clear
        };

        let mut gamma_correction = TextureTransformPipeline::new(
            &gamma_desc
        );

        let gamma_buffer = gamma_correction.spawn_framebuffer();


        let ambient_desc = TextureTransformDescriptor {
            render : render.clone(),
            format : wgpu::TextureFormat::Rgba32Float,
            size : extent,
            input_count : 5,
            output_count : 1,
            uniform : Some(Arc::new(AmbientLightUniform::default())),
            shader : include_str!("../../../../shaders/wgsl/ambient_light.wgsl").into(),
            blend : Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add
                }
            }),
            start_op : TextureTransformStart::None
        };

        let mut ambient_light_pipeline = TextureTransformPipeline::new(
            &ambient_desc
        );

        let gamma_buffer = gamma_correction.spawn_framebuffer();

        let ss_pipeline = crate::pipelines::wgpu_sreen_diffuse::SSDiffuse::new(
            &render,
            wgpu::Extent3d {
                width : extent.width,
                height : extent.height,
                depth_or_array_layers : 1
            },
            1,
            1,
            include_str!("../../../../shaders/wgsl/screen_diffuse_lighting.wgsl").into()
        );

        let ss_buffer = ss_pipeline.spawn_framebuffer();

        // let device_name = game.api.adapter.get_info().name;

        Self {
            present,
            render,
            gamma_correction,
            gamma_buffer,
            draw_state : DrawState::DirectLight,
            ambient_light : crate::light::AmbientLight {
                color : na::Vector3::new(1.0f32, 1.0, 1.0) * 0.05f32
            },
            ambient_light_pipeline
        }
    }
}

pub struct StateSystem {}

fn state_update(
    mut state : ResMut<State>,
    mut camera : Res<Camera>,
    mut lights : Query<&mut PointLight>
) {
    for mut light in &mut lights {
        light.update_buffer(&state.render);
    }

    state.render.device.poll(wgpu::Maintain::Wait);

    let ambient_uniform = AmbientLightUniform {
        color: state.ambient_light.color.into(),
        cam_pos: camera.pos.coords.clone()
    };
    state.ambient_light_pipeline.update(Some(&ambient_uniform));

    state.render.device.poll(wgpu::Maintain::Wait);
}

fn state_render(
    mut state : ResMut<State>,
    mut encoder : ResMut<RenderCommands>,
    render_target : Res<RenderTarget>,
    gbuffer : Res<GFramebuffer>,
    dir_light : Res<DirLightTexture>,
    ssao : Res<SSAOFiltered>
) {
        state.ambient_light_pipeline.draw(&mut encoder,
                                         &[&gbuffer.diffuse, &gbuffer.normal, &gbuffer.position, &gbuffer.mr, &ssao.tex]
                                         , &[&dir_light.tex]);
        

        let mut gamma_buffer = CommonFramebuffer {
            dst: vec![]
        };
        std::mem::swap(&mut gamma_buffer, &mut state.gamma_buffer);
        // game.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope("Final", encoder, &self.render.device);
        match &state.draw_state {
            DrawState::Full => {
                state.gamma_correction.draw(&mut encoder, &[&dir_light.tex], &[&gamma_buffer.dst[0]]);
                state.present.draw(&mut encoder, &gamma_buffer.dst[0], &render_target.view);
            }
            DrawState::DirectLight => {
                state.gamma_correction.draw(&mut encoder, &[&dir_light.tex], &[&gamma_buffer.dst[0]]);
                state.present.draw( &mut encoder, &gamma_buffer.dst[0], &render_target.view);
            },
            DrawState::AmbientOcclusion => {
                // state.gamma_correction.draw(&mut encoder, &[&game.scene.world.get_resource::<SSAOFrame>().unwrap().tex], &[&state.gamma_buffer.dst[0]]);
                // state.present.draw(&state.render.device, &mut encoder, &state.gamma_buffer.dst[0], &view);
            },
            DrawState::Depth => {
                // state.gamma_correction.draw(&mut encoder, &[&game.scene.world.get_resource::<DepthTexture>().unwrap().tex], &[&state.gamma_buffer.dst[0]]);
                // state.present.draw(&state.render.device, &mut encoder, &state.gamma_buffer.dst[0], &view);
            }
            DrawState::AmbientOcclusionSmooth => {
                state.gamma_correction.draw(&mut encoder, &[&ssao.tex], &[&gamma_buffer.dst[0]]);
                state.present.draw( &mut encoder, &gamma_buffer.dst[0], &render_target.view);
            }
        }
    std::mem::swap(&mut gamma_buffer, &mut state.gamma_buffer);
}

impl space_game::SchedulePlugin for StateSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("State".into())
    }

    fn add_system(&self, app: &mut space_core::app::App) {
        let state = pollster::block_on(State::new(app));
        
        app.add_system_to_stage(GlobalStageStep::PreRender, state_update);
        app.add_system_to_stage(GlobalStageStep::Render, state_render);
        app.insert_resource(state);
    }
}

impl space_game::RenderPlugin for State {
    fn update(&mut self, game : &mut Game) {
        self.render.device.poll(wgpu::Maintain::Wait);

        let ambient_uniform = AmbientLightUniform {
            color: self.ambient_light.color.into(),
            cam_pos: game.scene.app.world.get_resource::<Camera>().unwrap().pos.coords.clone()
        };
        self.ambient_light_pipeline.update(Some(&ambient_uniform));
    }

    fn render(&mut self, game : &mut Game) {
        let mut encoder = game.scene.app.world.remove_resource::<RenderCommands>().unwrap();
        let view = game.render_view.as_ref().unwrap();

        let mut light_queue = game.scene.app.world.query::<(&mut PointLight)>();
        for mut light in light_queue.iter_mut(&mut game.scene.app.world) {
            light.update_buffer(&self.render);
        }
        self.render.device.poll(wgpu::Maintain::Wait);

        let gbuffer = game.scene.app.world.get_resource::<GFramebuffer>().unwrap();
        // self.gbuffer_pipeline.draw(&game.assets, encoder, &mut game.scene.world, &self.gbuffer);
        // self.light_shadow.draw(encoder, &mut game.scene.world);


        // game.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope("Ambient", encoder, &self.render.device);
        // self.light_pipeline.draw(&self.render.device, encoder, &game.scene.world, &self.light_buffer, &gbuffer);
        self.ambient_light_pipeline.draw(&mut encoder,
                                         &[&gbuffer.diffuse, &gbuffer.normal, &gbuffer.position, &gbuffer.mr, &game.scene.app.world.get_resource::<SSAOFiltered>().unwrap().tex]
                                         , &[&game.scene.app.world.get_resource::<DirLightTexture>().unwrap().tex]);
        // game.scene.resources.get_mut::<GpuProfiler>().unwrap().end_scope(encoder);

        // game.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope("Final", encoder, &self.render.device);
        match &self.draw_state {
            DrawState::Full => {
                self.gamma_correction.draw(&mut encoder, &[&game.scene.app.world.get_resource::<DirLightTexture>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&mut encoder, &self.gamma_buffer.dst[0], &view);
            }
            DrawState::DirectLight => {
                self.gamma_correction.draw(&mut encoder, &[&game.scene.app.world.get_resource::<DirLightTexture>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&mut encoder, &self.gamma_buffer.dst[0], &view);
            },
            DrawState::AmbientOcclusion => {
                self.gamma_correction.draw(&mut encoder, &[&game.scene.app.world.get_resource::<SSAOFrame>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw( &mut encoder, &self.gamma_buffer.dst[0], &view);
            },
            DrawState::Depth => {
                self.gamma_correction.draw(&mut encoder, &[&game.scene.app.world.get_resource::<DepthTexture>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&mut encoder, &self.gamma_buffer.dst[0], &view);
            }
            DrawState::AmbientOcclusionSmooth => {
                self.gamma_correction.draw(&mut encoder, &[&game.scene.app.world.get_resource::<SSAOFiltered>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw( &mut encoder, &self.gamma_buffer.dst[0], &view);
            }
        }
        game.scene.app.insert_resource(encoder);
    }

    fn window_resize(&mut self, game : &mut Game, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            game.api.size = new_size;
            game.api.config.width = new_size.width;
            game.api.config.height = new_size.height;
            game.api.surface.configure(&self.render.device, &game.api.config);

            let size = wgpu::Extent3d {
                width : game.api.config.width,
                height : game.api.config.height,
                depth_or_array_layers : 1
            };

            self.present = TexturePresent::new(
                &self.render,
                game.api.config.format,
                size);



            let mut gamma_desc = self.gamma_correction.get_desc();
            gamma_desc.size = size;
            self.gamma_correction = TextureTransformPipeline::new(
                &gamma_desc
            );


            self.gamma_buffer = self.gamma_correction.spawn_framebuffer();

            let mut ambient_desc = self.ambient_light_pipeline.get_desc();
            ambient_desc.size = size;
            self.ambient_light_pipeline = TextureTransformPipeline::new(
                &ambient_desc
            );
        }
    }

    fn show_top_panel(&mut self, game: &mut Game, ui: &mut Ui) {
        egui::ComboBox::from_label("Draw mode")
            .selected_text(format!("{:?}", &self.draw_state))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.draw_state, DrawState::DirectLight, "DirectLight");
                ui.selectable_value(&mut self.draw_state, DrawState::AmbientOcclusion, "AmbientOcclusion");
                ui.selectable_value(&mut self.draw_state, DrawState::AmbientOcclusionSmooth, "AmbientOcclusionSmooth");
                ui.selectable_value(&mut self.draw_state, DrawState::Depth, "Depth");
            });
    }
}