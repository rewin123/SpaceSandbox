use std::iter;
use std::ops::Deref;
use std::sync::Arc;

use SpaceSandbox::ui::{FpsCounter};
use bytemuck::{Zeroable, Pod};
use egui::epaint::ahash::HashMap;
use egui_gizmo::GizmoMode;
use egui_wgpu_backend::ScreenDescriptor;
use space_render::pipelines::wgpu_sreen_diffuse::SSDiffuse;
use space_shaders::*;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use SpaceSandbox::{init_logger};
use encase::{ShaderType, UniformBuffer};
use image::gif::Encoder;
use space_assets::*;

use nalgebra as na;
use nalgebra::Matrix4;
use wgpu::{BlendFactor, MaintainBase};
use space_core::{RenderBase, TaskServer};
use space_render::{pipelines::*, Camera};
use space_render::light::*;
use space_render::pipelines::wgpu_ssao::SSAO;

use legion::*;
use space_game::{Game, RenderPlugin};

#[repr(C)]
#[derive(Clone, Zeroable, Pod, Copy)]
pub struct SmoothUniform {
    size : [i32; 2]
}

impl TextureTransformUniform for SmoothUniform {
    fn get_bytes(&self) -> Vec<u8> {
        let bytes = bytemuck::bytes_of(self);
        bytes.to_vec()
    }
}

async fn run() {
    init_logger();
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new().await;

    let mut game = state.game.take().unwrap();
    game.add_render_plugin(state);

    game.run();
}

#[derive(Debug, PartialEq)]
enum DrawState {
    Full,
    DirectLight,
    AmbientOcclusion,
    Depth
}

struct State {
    game : Option<Game>,
    render : Arc<RenderBase>,
    scene : World,
    camera : Camera,
    camera_buffer : wgpu::Buffer,
    gbuffer_pipeline : GBufferFill,
    light_shadow : PointLightShadowPipeline,

    ss_diffuse : SSDiffuse,
    ss_difuse_framebufer : CommonFramebuffer,

    light_pipeline : PointLightPipeline,
    gamma_correction : TextureTransformPipeline,
    depth_calc : TextureTransformPipeline,
    depth_buffer : CommonFramebuffer,
    light_buffer : TextureBundle,
    gbuffer : GFramebuffer,
    present : TexturePresent,
    gamma_buffer : CommonFramebuffer,
    point_lights : Vec<PointLight>,
    fps : FpsCounter,
    device_name : String,

    draw_state : DrawState,
    ambient_light : AmbientLight,
    ambient_light_pipeline : TextureTransformPipeline
}




#[derive(Default)]
struct DepthCalcUniform {
    pub cam_pos : [f32; 4]
}

impl TextureTransformUniform for DepthCalcUniform {
    fn get_bytes(&self) -> Vec<u8> {
        bytemuck::cast_slice(&self.cam_pos).to_vec()
    }
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new() -> Self {
        let game = Game::default();
        let render = game.get_render_base();


        let camera = Camera::default();
        let camera_uniform = camera.build_uniform();

        let mut camera_cpu_buffer = UniformBuffer::new(vec![0u8;100]);
        camera_cpu_buffer.write(&camera_uniform);

        let camera_buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label : Some("Camera uniform buffer"),
            contents : &camera_cpu_buffer.into_inner(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let extent = wgpu::Extent3d {
            width : game.api.config.width,
            height : game.api.config.height,
            depth_or_array_layers : 1
        };

        let mut world = World::default();

        let task_server = Arc::new(TaskServer::new());

        let mut assets = AssetServer::new(&render, &task_server);

        assets.wgpu_gltf_load(
            &render.device,
            "res/test_res/models/sponza/glTF/Sponza.gltf".into(),
            &mut world);

        // assets.wgpu_gltf_load(
        //     &render.device,
        //     "res/bobik/bobik.gltf".into(),
        //     &mut world);

        let gbuffer = GBufferFill::new(
            &render,
            &camera_buffer,
            game.api.config.format,
            wgpu::Extent3d {
                width : game.api.config.width,
                height : game.api.config.height,
                depth_or_array_layers : 1
            });

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

        let mut lights = vec![
            PointLight::new(&render, [0.0, 3.0, 0.0].into(), true),
            // PointLight::new(&render, [0.0, 1.0, 0.0].into(), true),
        ];
        lights[0].intensity = 20.0;
        // lights[1].intensity = 1.0;

        let point_light_shadow = PointLightShadowPipeline::new(&render);

        let light_pipeline = PointLightPipeline::new(&render, &camera_buffer, extent);
        let light_buffer = light_pipeline.spawn_framebuffer(&render.device, extent);



        let fps = FpsCounter::default();

        let gamma_desc = TextureTransformDescriptor {
            render : render.clone(),
            format: wgpu::TextureFormat::Rgba32Float,
            size: extent,
            input_count: 1,
            output_count: 1,
            uniform: None,
            shader: include_str!("../shaders/wgsl/gamma_correction.wgsl").into(),
            blend : None,
            start_op : TextureTransformStart::Clear
        };

        let mut gamma_correction = TextureTransformPipeline::new(
            &gamma_desc
        );

        let gamma_buffer = gamma_correction.spawn_framebuffer();

        let depth_desc = TextureTransformDescriptor {
            render : render.clone(),
            format : wgpu::TextureFormat::R16Float,
            size : extent,
            input_count : 1,
            output_count : 1,
            uniform : Some(Arc::new(DepthCalcUniform::default())),
            shader : include_str!("../shaders/wgsl/depth_calc.wgsl").into(),
            blend : None,
            start_op : TextureTransformStart::Clear
        };

        let mut depth_calc = TextureTransformPipeline::new(
            &depth_desc
        );

        let ambient_desc = TextureTransformDescriptor {
            render : render.clone(),
            format : wgpu::TextureFormat::Rgba32Float,
            size : extent,
            input_count : 5,
            output_count : 1,
            uniform : Some(Arc::new(AmbientLightUniform::default())),
            shader : include_str!("../shaders/wgsl/ambient_light.wgsl").into(),
            blend : Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: wgpu::BlendOperation::Add
                },
                alpha: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: wgpu::BlendOperation::Add
                }
            }),
            start_op : TextureTransformStart::None
        };

        let mut ambient_light_pipeline = TextureTransformPipeline::new(
            &ambient_desc
        );

        let depth_buffer = depth_calc.spawn_framebuffer();

        let gamma_buffer = gamma_correction.spawn_framebuffer();

        let ss_pipeline = SSDiffuse::new(
            &render,
            wgpu::Extent3d {
                width : extent.width,
                height : extent.height,
                depth_or_array_layers : 1
            },
            1,
            1,
            include_str!("../shaders/wgsl/screen_diffuse_lighting.wgsl").into()
        );

        let ss_buffer = ss_pipeline.spawn_framebuffer();

        let device_name = game.api.adapter.get_info().name;

        Self {
            game : Some(game),
            scene : world,
            camera : Camera::default(),
            camera_buffer,
            gbuffer_pipeline : gbuffer,
            gbuffer : framebuffer,
            present,
            point_lights : lights,
            light_pipeline,
            light_buffer,
            assets,
            render,
            fps,
            light_shadow : point_light_shadow,
            gamma_correction,
            gamma_buffer,
            device_name,
            ss_diffuse : ss_pipeline,
            ss_difuse_framebufer : ss_buffer,
            draw_state : DrawState::DirectLight,
            depth_calc,
            depth_buffer,
            ambient_light : AmbientLight {
                color : na::Vector3::new(1.0f32, 1.0, 1.0) * 0.1f32
            },
            ambient_light_pipeline
        }
    }
}

impl RenderPlugin for State {
    fn update(&mut self, game : &mut Game) {
        let speed = 0.3 / 5.0;
        if game.input.get_key_state(VirtualKeyCode::W) {
            self.camera.pos += self.camera.frw * speed;
        }
        if game.input.get_key_state(VirtualKeyCode::S) {
            self.camera.pos -= self.camera.frw * speed;
        }
        if game.input.get_key_state(VirtualKeyCode::D) {
            self.camera.pos += self.camera.get_right() * speed;
        }
        if game.input.get_key_state(VirtualKeyCode::A) {
            self.camera.pos -= self.camera.get_right() * speed;
        }
        if game.input.get_key_state(VirtualKeyCode::Space) {
            self.camera.pos += self.camera.up  * speed;
        }
        if game.input.get_key_state(VirtualKeyCode::LShift) {
            self.camera.pos -= self.camera.up * speed;
        }

        let mut loc_query = <(&mut Location,)>::query();

        self.ss_diffuse.update(&self.camera);
        for loc in loc_query.iter_mut(&mut self.scene) {
            loc.0.update_buffer();
        }
        self.render.device.poll(wgpu::Maintain::Wait);

        let camera_unifrom = self.camera.build_uniform();
        let mut uniform = encase::UniformBuffer::new(vec![]);
        uniform.write(&camera_unifrom).unwrap();
        let inner = uniform.into_inner();

        let tmp_buffer = self.render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &inner,
            usage: wgpu::BufferUsages::COPY_SRC,
        });

        let depth_buffer = DepthCalcUniform {
            cam_pos : [self.camera.pos.x, self.camera.pos.y, self.camera.pos.z, 1.0]
        };

        self.depth_calc.update(Some(&depth_buffer));

        let ambient_uniform = AmbientLightUniform {
            color: self.ambient_light.color.into(),
            cam_pos: self.camera.pos.coords.clone()
        };
        self.ambient_light_pipeline.update(Some(&ambient_uniform));


        let mut encoder = self
        .render.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Update encoder"),
            });

        encoder.copy_buffer_to_buffer(
            &tmp_buffer,
            0,
            &self.camera_buffer,
            0,
            inner.len() as wgpu::BufferAddress);
        self.render.queue.submit(iter::once(encoder.finish()));
    }

    fn render(&mut self, game : &mut Game, encoder : &mut wgpu::CommandEncoder) {
        let view = game.render_view.as_ref().unwrap();

        for light in &mut self.point_lights {
            light.update_buffer(&self.render);
        }
        self.render.device.poll(wgpu::Maintain::Wait);

        self.gbuffer_pipeline.draw(&self.assets, encoder, &mut self.scene, &self.gbuffer);
        self.depth_calc.draw(encoder, &[&self.gbuffer.position], &[&self.depth_buffer.dst[0]]);
        self.light_shadow.draw(encoder, &mut self.point_lights, &self.scene);
        self.ss_diffuse.draw(
            encoder,
            &self.gbuffer,
            &self.light_buffer,
            &self.depth_buffer.dst[0],
            &self.ss_difuse_framebufer.dst[0]);

        self.light_pipeline.draw(&self.render.device, encoder, &self.point_lights, &self.light_buffer, &self.gbuffer);
        self.ambient_light_pipeline.draw(encoder,
            &[&self.gbuffer.diffuse, &self.gbuffer.normal, &self.gbuffer.position, &self.gbuffer.mr, &self.ss_difuse_framebufer.dst[0]]
        , &[&self.light_buffer]);
        // self.gamma_correction.draw(&self.render.device, &mut encoder, &[&self.light_buffer], &[&self.gamma_buffer.dst[0]]);

        match &self.draw_state {
            DrawState::Full => {
                self.gamma_correction.draw(encoder, &[&self.light_buffer], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            }
            DrawState::DirectLight => {
                self.gamma_correction.draw(encoder, &[&self.light_buffer], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            },
            DrawState::AmbientOcclusion => {
                self.gamma_correction.draw(encoder, &[&self.ss_difuse_framebufer.dst[0]], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            },
            DrawState::Depth => {
                self.gamma_correction.draw(encoder, &[&self.depth_buffer.dst[0]], &[&self.gamma_buffer.dst[0]]);
                self.present.draw(&self.render.device, encoder, &self.gamma_buffer.dst[0], &view);
            }
        }
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
                            ui.selectable_value(&mut self.draw_state, DrawState::Depth, "Depth");
                        });

                    self.fps.draw(ui);
                    ui.label(&self.device_name);
                });

                let cam_uniform = self.camera.build_uniform();
                let gizmo = egui_gizmo::Gizmo::new("light gizmo").projection_matrix(
                    cam_uniform.proj
                ).view_matrix(cam_uniform.view)
                    .model_matrix(na::Matrix4::new_translation(&self.point_lights[0].pos))
                    .mode(GizmoMode::Translate);

                if let Some(responce) = gizmo.interact(ui) {
                    let mat : Matrix4<f32> = responce.transform.into();
                    self.point_lights[0].pos.x = mat.m14;
                    self.point_lights[0].pos.y = mat.m24;
                    self.point_lights[0].pos.z = mat.m34;

                }


        });

        let gui_output = game.gui.end_frame(Some(&game.window));
        game.gui.draw(gui_output,
            ScreenDescriptor {
                physical_width: game.api.config.width,
                physical_height: game.api.config.height,
                scale_factor: game.window.scale_factor() as f32,
            },
            encoder,
            &view);

        self.assets.sync_tick();
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

            self.gbuffer_pipeline = GBufferFill::new(
                &self.render,
                &self.camera_buffer,
                game.api.config.format,
                size.clone()
            );

            self.gbuffer = GBufferFill::spawn_framebuffer(
                &self.render.device,
            size.clone());

            self.present = TexturePresent::new(
                &self.render.device,
                game.api.config.format,
                size);

            self.light_pipeline = PointLightPipeline::new(
                &self.render,
                &self.camera_buffer,
                size
            );

            let mut gamma_desc = self.gamma_correction.get_desc();
            gamma_desc.size = size;
            self.gamma_correction = TextureTransformPipeline::new(
                &gamma_desc
            );


            self.gamma_buffer = self.gamma_correction.spawn_framebuffer();

            self.light_buffer = self.light_pipeline.spawn_framebuffer(&self.render.device, size);

            self.ss_diffuse = SSDiffuse::new(
                &self.render,
                size,
                1,
                1,
                include_str!("../shaders/wgsl/screen_diffuse_lighting.wgsl").into()
            );

            self.ss_difuse_framebufer = self.ss_diffuse.spawn_framebuffer();

            let mut depth_desc = self.depth_calc.get_desc();
            depth_desc.size = size;
            self.depth_calc = TextureTransformPipeline::new(
                &depth_desc
            );

            self.depth_buffer = self.depth_calc.spawn_framebuffer();

            let mut ambient_desc = self.ambient_light_pipeline.get_desc();
            ambient_desc.size = size;
            self.ambient_light_pipeline = TextureTransformPipeline::new(
                &ambient_desc
            );
        }
    }
}

fn main() {
    pollster::block_on(run());
}