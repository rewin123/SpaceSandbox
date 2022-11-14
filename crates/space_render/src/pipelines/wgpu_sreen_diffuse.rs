use std::fs::rename;
use std::num::NonZeroU32;
use std::sync::Arc;
use bytemuck::Zeroable;
use rand::Rng;
use space_game::{SchedulePlugin, PluginName, GlobalStageStep, ScreenSize, RenderApi, RenderCommands};
use wgpu::{Extent3d, util::DeviceExt};
use space_assets::*;
use space_core::{Camera, RenderBase, app::App};
use crate::{pipelines::{CommonFramebuffer, GFramebuffer}};
use encase::*;
use wgpu_profiler::GpuProfiler;

use space_core::ecs::*;
use super::{wgpu_ssao::SSAOFrame, DirLightTexture};

#[repr(C)]
#[derive(Zeroable, bytemuck::Pod, Clone, Copy)]
struct ScreenDiffuseUniform {
    proj_view : [[f32; 4]; 4],
    proj_view_inverse : [[f32; 4]; 4],
    random_vecs : [[f32; 4]; 64],
    cam_pos : [f32; 4],
    tex_width : f32,
    tex_height : f32,
    scale : f32,
    dummy1 : f32
}

#[derive(Resource)]
pub struct DepthTexture {
    pub tex : TextureBundle
}

fn ssao_impl( 
    mut ssao_pipeline : ResMut<SSDiffuse>,
    mut encoder : ResMut<RenderCommands>,
    gbuffer : Res<GFramebuffer>,
    ssao_frame : Res<SSAOFrame>,
    dir_light : Res<DirLightTexture>,
    depth : Res<DepthTexture>) {

    // profiler.begin_scope("SSAO", encoder, &ssao_pipeline.render.device);
    ssao_pipeline.draw(encoder.as_mut(), gbuffer.as_ref(), &dir_light.tex, &depth.tex, &ssao_frame.tex);
    // profiler.end_scope(encoder);
}

fn ssao_update(
    mut ssao_pipeline : ResMut<SSDiffuse>,
    mut camera : Res<Camera>,
) {
    ssao_pipeline.update(camera.as_ref());
}

pub struct SSDiffuseSystem {

}

impl SchedulePlugin for SSDiffuseSystem {
    fn get_name(&self) -> space_game::PluginName {
        PluginName::Text("SSAO".into())
    }

    fn add_system(&self, app : &mut App) {
        
        let render = app.world.get_resource::<RenderApi>().unwrap().base.clone();
        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();

        let pipeline = SSDiffuse::new(
            &render, 
            wgpu::Extent3d {
                width : size.width / 2,
                height : size.height / 2,
                depth_or_array_layers : 1,
            }, 
            1, 
            1, 
            include_str!("../../../../shaders/wgsl/screen_diffuse_lighting.wgsl").into());

        let frame = pipeline.spawn_framebuffer();
        
        app.insert_resource(frame);
        app.insert_resource(pipeline);

        app.add_system_to_stage(GlobalStageStep::Render, ssao_impl);
        app.add_system_to_stage(GlobalStageStep::PreRender, ssao_update);
    }
}

impl Default for ScreenDiffuseUniform {
    fn default() -> Self {

        let mut rnd = rand::thread_rng();

        let mut random_vecs = [[0.0f32; 4]; 64];

        for idx in 0..64 {
            let mut v = nalgebra::Vector3::new(
                rnd.gen_range(-1.0..=1.0f32),
                rnd.gen_range(-1.0..=1.0f32),
                rnd.gen_range(0.01..=1.0f32)).normalize();

            let scale = (idx as f32) / 64.0;
            v = v * SSDiffuse::lerp(scale * scale, 0.1, 1.0);

            random_vecs[idx][0] = v.x;
            random_vecs[idx][1] = v.y;
            random_vecs[idx][2] = v.z;
            random_vecs[idx][3] = 1.0;
        }

        Self {
            proj_view : [[0.0; 4]; 4],
            proj_view_inverse : [[0.0; 4]; 4],
            random_vecs,
            cam_pos : [0.0; 4],
            tex_width : 0.0,
            tex_height : 0.0,
            scale : 1.0,
            dummy1 : 0.0
        }
    }
}

#[derive(Resource)]
pub struct SSDiffuse {
    pub pipeline : wgpu::RenderPipeline,
    screen_mesh : ScreenMesh,
    texture_bind_group_layout : wgpu::BindGroupLayout,
    output_format : wgpu::TextureFormat,
    output_count : u32,
    input_count : u32,
    size : Extent3d,
    render : Arc<RenderBase>,
    bind: Option<wgpu::BindGroup>,
    buffer : Arc<wgpu::Buffer>,
    pub scale : f32,
    cached_uniform : ScreenDiffuseUniform,
    noise_texture : TextureBundle
}

impl SSDiffuse {

    pub fn spawn_framebuffer(&self) -> SSAOFrame {
        SSAOFrame {
            tex : TextureBundle::new(&self.render.device, &wgpu::TextureDescriptor {
                label: None,
                size: self.size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.output_format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
            }, wgpu::FilterMode::Nearest)
        }
    }
    
    pub fn spawn_renderpass<'a>(&'a self, encoder : &'a mut wgpu::CommandEncoder, dst : &'a TextureBundle) -> wgpu::RenderPass {

        let mut attachs = vec![];
        attachs.push(Some(wgpu::RenderPassColorAttachment {
            view: &dst.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                }),
                store: true,
            },
        }));

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Texture present render Pass"),
            color_attachments: &attachs,
            depth_stencil_attachment: None,
        });

        render_pass
    }

    fn lerp(val : f32, start : f32, end : f32) -> f32 {
        start + val * (end - start)
    }

    fn create_noise_tex(render : &Arc<RenderBase>) -> TextureBundle {
        let size = 128;

        let mut rnd = rand::thread_rng();

        let data : Vec<u8> = (0..(size * size * 4)).map(|idx| {rnd.gen::<u8>()}).collect();

        let tex = render.device.create_texture_with_data(&render.queue, 
            &wgpu::TextureDescriptor {
                label: Some("Noise texture"),
                size: wgpu::Extent3d {
                    width: size,
                    height: size,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Snorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
            }, 
        &data);

        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        TextureBundle { texture: tex, view, sampler }
    }

    pub fn new(
        render : &Arc<RenderBase>,
        size : wgpu::Extent3d,
        input_count : u32,
        output_count : u32,
        shader : String) -> Self {

        let noise = SSDiffuse::create_noise_tex(render);

        let format = wgpu::TextureFormat::Rgba32Float;
        let input_count = 5;
        let mut binds = vec![];
        for idx in 0..input_count {
            binds.push(wgpu::BindGroupLayoutEntry {
                binding: idx * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false
                },
                count: None
            });
            binds.push(wgpu::BindGroupLayoutEntry {
                binding : idx * 2 + 1,
                visibility : wgpu::ShaderStages::FRAGMENT,
                ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count : None
            });
        }
        binds.push(wgpu::BindGroupLayoutEntry {
            binding: binds.len() as u32,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer { 
                ty: wgpu::BufferBindingType::Uniform, 
                has_dynamic_offset: false, 
                min_binding_size: None 
            },
            count: None,
        });

        let texture_bind_group_layout = render.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label : Some("Texture present binding"),
            entries : &binds
        });

        let shader = render.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader.into())
        });

        let pipeline_layout =
            render.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Texture transform"),
                bind_group_layouts : &[&texture_bind_group_layout],
                push_constant_ranges: &[]
            });

        let pipeline = render.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers: &[SimpleVertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(wgpu::ColorTargetState {
                    format,
                    blend : Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask : wgpu::ColorWrites::ALL
                }),]
            }),
            primitive: wgpu::PrimitiveState {
                topology : wgpu::PrimitiveTopology::TriangleList,
                strip_index_format : None,
                front_face : wgpu::FrontFace::Ccw,
                cull_mode : None,
                polygon_mode : wgpu::PolygonMode::Fill,
                unclipped_depth : false,
                conservative : false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None
        });

        let mut def_uniform = ScreenDiffuseUniform::default();

        let buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&def_uniform),
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::UNIFORM,
        });

        Self {
            pipeline,
            screen_mesh : SSDiffuse::create_screen_mesh(&render.device),
            texture_bind_group_layout,
            output_format : format,
            output_count,
            input_count,
            size,
            render: render.clone(),
            bind: None,
            buffer : Arc::new(buffer),
            scale : 1.0,
            cached_uniform : def_uniform,
            noise_texture : noise
        }
    }

    pub fn update(&mut self, camera : &Camera) {
        let cam_uniform = camera.build_uniform();

        let proj_view = cam_uniform.proj * cam_uniform.view;

        self.cached_uniform.proj_view = proj_view.into();
        self.cached_uniform.proj_view_inverse = proj_view.try_inverse().unwrap().into();
        self.cached_uniform.tex_width = self.size.width as f32;
        self.cached_uniform.tex_height = self.size.height as f32;
        self.cached_uniform.scale = self.scale;
        self.cached_uniform.cam_pos = [camera.pos.x, camera.pos.y, camera.pos.z, 1.0];

        let ssao = self.cached_uniform.clone();
        

        let buffer = self.buffer.clone();

        self.buffer.slice(..).map_async(wgpu::MapMode::Write,  move |a| {
            buffer.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::bytes_of(&ssao));
            buffer.unmap();
        });
    }

    pub fn draw(
            &mut self,
            encoder : &mut wgpu::CommandEncoder,
            src : &GFramebuffer,
            dir_light : &TextureBundle,
            depth : &TextureBundle,
            dst : &TextureBundle) {

        let mut binds = vec![];
        binds.push(
            wgpu::BindGroupEntry {
                binding : 0,
                resource : wgpu::BindingResource::TextureView(&src.normal.view)
            },
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 1,
                resource : wgpu::BindingResource::Sampler(&src.normal.sampler)
            }
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 2,
                resource : wgpu::BindingResource::TextureView(&src.position.view)
            },
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 3,
                resource : wgpu::BindingResource::Sampler(&src.position.sampler)
            }
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 4,
                resource : wgpu::BindingResource::TextureView(&dir_light.view)
            },
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 5,
                resource : wgpu::BindingResource::Sampler(&dir_light.sampler)
            }
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 6,
                resource : wgpu::BindingResource::TextureView(&depth.view)
            },
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 7,
                resource : wgpu::BindingResource::Sampler(&depth.sampler)
            }
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 8,
                resource : wgpu::BindingResource::TextureView(&self.noise_texture.view)
            },
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 9,
                resource : wgpu::BindingResource::Sampler(&self.noise_texture.sampler)
            }
        );
        binds.push(
            wgpu::BindGroupEntry {
                binding : 10,
                resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &self.buffer,
                    offset: 0,
                    size: None,
                })
            }
        );
        
        let tex_bind = self.render.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &self.texture_bind_group_layout,
            entries : &binds,
            label : Some("texture present bind")
        });

        let mut render_pass = self.spawn_renderpass(encoder, dst);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &tex_bind, &[]);
        render_pass.set_vertex_buffer(0, self.screen_mesh.vertex.slice(..));
        render_pass.draw(0..6, 0..1);
    }

    fn create_screen_mesh(device : &wgpu::Device) -> ScreenMesh {
        let vertex = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Screen vertex"),
            contents: bytemuck::cast_slice(&[
                SimpleVertex {
                    pos: [-1.0, -1.0, 0.0],
                },
                SimpleVertex {
                    pos: [-1.0, 1.0, 0.0],
                },
                SimpleVertex {
                    pos: [1.0, -1.0, 0.0],
                },
                SimpleVertex {
                    pos: [1.0, -1.0, 0.0],
                },
                SimpleVertex {
                    pos: [-1.0, 1.0, 0.0],
                },
                SimpleVertex {
                    pos: [1.0, 1.0, 0.0],
                },
            ]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
        });

        ScreenMesh { 
            vertex 
        }
    }
}

struct ScreenMesh {
    pub vertex : wgpu::Buffer,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SimpleVertex {
    pub pos : [f32; 3]
}

impl SimpleVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}