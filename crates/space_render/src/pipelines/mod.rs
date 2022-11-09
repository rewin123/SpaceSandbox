use std::{fmt::Debug, sync::Arc};
use std::ops::DerefMut;
use downcast_rs::{Downcast, impl_downcast};
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

use space_core::{Camera, RenderBase};
use space_game::{Game, PluginName, PluginType, SchedulePlugin};
pub use wgpu_gbuffer_fill::*;
pub use wgpu_light_fill::*;
use wgpu_profiler::GpuProfiler;
pub use wgpu_texture_present::*;
pub use wgpu_light_shadow::*;
pub use wgpu_textures_transform::*;

use legion::*;
use legion::systems::Builder;
use crate::light::{AmbientLightUniform, PointLight};
use crate::pipelines::wgpu_ssao::SSAOFrame;
use crate::ui::FpsCounter;

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


#[derive(Debug, PartialEq)]
enum DrawState {
    Full,
    DirectLight,
    AmbientOcclusion,
    AmbientOcclusionSmooth,
    Depth
}

pub struct State {
    pub game : Option<Game>,
    render : Arc<RenderBase>,

    gamma_correction : TextureTransformPipeline,
    present : TexturePresent,
    gamma_buffer : CommonFramebuffer,
    fps : crate::ui::FpsCounter,
    device_name : String,

    draw_state : DrawState,
    ambient_light : crate::light::AmbientLight,
    ambient_light_pipeline : TextureTransformPipeline
}


impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new() -> Self {
        let mut game = Game::default();
        let render = game.get_render_base();

        let extent = wgpu::Extent3d {
            width : game.api.config.width,
            height : game.api.config.height,
            depth_or_array_layers : 1
        };


        let framebuffer = GBufferFill::spawn_framebuffer(
            &render.device,
            extent);

        let present = TexturePresent::new(
            &render.device,
            game.api.config.format,
            wgpu::Extent3d {
                width : game.api.config.width,
                height : game.api.config.height,
                depth_or_array_layers : 1
            });
        let point_light_shadow = PointLightShadowPipeline::new(&render);

        let light_pipeline = PointLightPipeline::new(&render, &game.scene.camera_buffer, extent);
        let light_buffer = light_pipeline.spawn_framebuffer(&render.device, extent);

        let fps = FpsCounter::default();

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

        let device_name = game.api.adapter.get_info().name;

        Self {
            game : Some(game),
            present,
            render,
            fps,
            gamma_correction,
            gamma_buffer,
            device_name,
            draw_state : DrawState::DirectLight,
            ambient_light : crate::light::AmbientLight {
                color : na::Vector3::new(1.0f32, 1.0, 1.0) * 0.05f32
            },
            ambient_light_pipeline
        }
    }
}

impl space_game::RenderPlugin for State {
    fn update(&mut self, game : &mut Game) {


        // let mut loc_query = <(&mut Location,)>::query();

        // for loc in loc_query.iter_mut(&mut game.scene.world) {
        //     loc.0.update_buffer();
        // }
        self.render.device.poll(wgpu::Maintain::Wait);

        let ambient_uniform = AmbientLightUniform {
            color: self.ambient_light.color.into(),
            cam_pos: game.scene.camera.pos.coords.clone()
        };
        self.ambient_light_pipeline.update(Some(&ambient_uniform));
    }

    fn render(&mut self, game : &mut Game) {
        let mut encoder_ref = game.scene.resources.get_mut::<wgpu::CommandEncoder>().unwrap();
        let encoder = encoder_ref.deref_mut();
        let view = game.render_view.as_ref().unwrap();

        let mut light_queue = <(&mut PointLight)>::query();
        for light in light_queue.iter_mut(&mut game.scene.world) {
            light.update_buffer(&self.render);
        }
        self.render.device.poll(wgpu::Maintain::Wait);

        let gbuffer = game.scene.resources.get::<GFramebuffer>().unwrap();
        // self.gbuffer_pipeline.draw(&game.assets, encoder, &mut game.scene.world, &self.gbuffer);
        // self.light_shadow.draw(encoder, &mut game.scene.world);


        game.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope("Ambient", encoder, &self.render.device);
        // self.light_pipeline.draw(&self.render.device, encoder, &game.scene.world, &self.light_buffer, &gbuffer);
        self.ambient_light_pipeline.draw(encoder,
                                         &[&gbuffer.diffuse, &gbuffer.normal, &gbuffer.position, &gbuffer.mr, &game.scene.resources.get::<SSAOFiltered>().unwrap().tex]
                                         , &[&game.scene.resources.get::<DirLightTexture>().unwrap().tex]);
        game.scene.resources.get_mut::<GpuProfiler>().unwrap().end_scope(encoder);

        game.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope("Final", encoder, &self.render.device);
        match &self.draw_state {
            DrawState::Full => {
                self.gamma_correction.draw(encoder, &[&game.scene.resources.get::<DirLightTexture>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            }
            DrawState::DirectLight => {
                self.gamma_correction.draw(encoder, &[&game.scene.resources.get::<DirLightTexture>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            },
            DrawState::AmbientOcclusion => {
                self.gamma_correction.draw(encoder, &[&game.scene.resources.get::<SSAOFrame>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            },
            DrawState::Depth => {
                self.gamma_correction.draw(encoder, &[&game.scene.resources.get::<DepthTexture>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            }
            DrawState::AmbientOcclusionSmooth => {
                self.gamma_correction.draw(encoder, &[&game.scene.resources.get::<SSAOFiltered>().unwrap().tex], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            }
        }
        game.scene.resources.get_mut::<GpuProfiler>().unwrap().end_scope(encoder);
        // self.present.draw(&self.render.device, &mut encoder, &self.ssao_smooth_framebuffer.dst[0], &view);

        game.gui.begin_frame();

        egui::TopBottomPanel::top("top_panel").show(
            &game.gui.platform.context(), |ui| {

                ui.horizontal(|ui| {

                    egui::ComboBox::from_label("Draw mode")
                        .selected_text(format!("{:?}", &self.draw_state))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.draw_state, DrawState::DirectLight, "DirectLight");
                            ui.selectable_value(&mut self.draw_state, DrawState::AmbientOcclusion, "AmbientOcclusion");
                            ui.selectable_value(&mut self.draw_state, DrawState::AmbientOcclusionSmooth, "AmbientOcclusionSmooth");
                            ui.selectable_value(&mut self.draw_state, DrawState::Depth, "Depth");
                        });

                    self.fps.draw(ui);
                    ui.label(&self.device_name);
                });

                // let cam_uniform = self.camera.build_uniform();
                // let gizmo = egui_gizmo::Gizmo::new("light gizmo").projection_matrix(
                //     cam_uniform.proj
                // ).view_matrix(cam_uniform.view)
                //     .model_matrix(na::Matrix4::new_translation(&self.point_lights[0].pos))
                //     .mode(GizmoMode::Translate);
                //
                // if let Some(responce) = gizmo.interact(ui) {
                //     let mat : Matrix4<f32> = responce.transform.into();
                //     self.point_lights[0].pos.x = mat.m14;
                //     self.point_lights[0].pos.y = mat.m24;
                //     self.point_lights[0].pos.z = mat.m34;
                // }
            });

        let gui_output = game.gui.end_frame(Some(&game.window));
        game.scene.resources.get_mut::<GpuProfiler>().unwrap().begin_scope("Gui", encoder, &self.render.device);
        game.gui.draw(gui_output,
                      egui_wgpu_backend::ScreenDescriptor {
                          physical_width: game.api.config.width,
                          physical_height: game.api.config.height,
                          scale_factor: game.window.scale_factor() as f32,
                      },
                      encoder,
                      &view);
        game.scene.resources.get_mut::<GpuProfiler>().unwrap().end_scope(encoder);
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
                &self.render.device,
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
}