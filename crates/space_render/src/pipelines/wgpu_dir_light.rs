use std::num::NonZeroU32;
use std::sync::Arc;
use legion::world::SubWorld;
use wgpu::{Buffer, Extent3d, TextureDimension};
use crate::light::PointLight;
use downcast_rs::*;
use legion::{IntoQuery, World};
use legion::systems::Builder;
use space_assets::*;
use space_assets::wavefront::wgpu_load_gray_obj;
use space_core::{RenderBase, ScreenMesh};
use space_game::{Game, PluginName, PluginType, SchedulePlugin};
use crate::pipelines::{DirLightTexture, Pipeline, PipelineDesc, TextureTransformPipeline};
use crate::pipelines::wgpu_gbuffer_fill::GFramebuffer;
use legion::*;
use wgpu_profiler::GpuProfiler;
use nalgebra as na;
use encase::*;
use wgpu::util::DeviceExt;

#[derive(ShaderType, Default)]
pub struct DirLightUniform {
    pub dir : na::Vector3<f32>,
    pub color : na::Vector3<f32>,
    pub intensity : f32,
}


pub struct DirLight {
    pub dir : nalgebra::Vector3<f32>,
    pub color : nalgebra::Vector3<f32>,
    pub intesity : f32,
    pub buffer : Arc<wgpu::Buffer>
}

impl DirLight {
    pub fn default(render : &Arc<RenderBase>) -> Self {
        let dir = -na::Vector3::new(0.0f32, 1.0f32, 1.0).normalize();
        let color = na::Vector3::new(253.0f32, 184.0, 19.0) / 255.0f32;
        let intesity = 1.0f32;

        let mut uniform_struct = DirLightUniform::default();
        uniform_struct.color = color;
        uniform_struct.dir = dir;
        uniform_struct.intensity = intesity;

        let mut uniform = UniformBuffer::new(vec![]);
        uniform.write(&uniform_struct).unwrap();

        let buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dir light uniform"),
            contents: &uniform.into_inner(),
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::UNIFORM
        });

        Self {
            dir,
            color,
            intesity,
            buffer : Arc::new(buffer)
        }
    }
}

#[derive(Clone, Debug)]
pub struct DirLightPipelineDesc {
    shader_path : AssetPath,
    render : Arc<RenderBase>,
    size : wgpu::Extent3d
}

impl PipelineDesc for DirLightPipelineDesc {
    fn get_shader_path(&self) -> AssetPath {
        AssetPath::Text("".into())
    }
    fn set_shader_path(&mut self, path: AssetPath) {
        self.shader_path = path;
    }
    fn clone_boxed(&self) -> Box<dyn PipelineDesc> {
        Box::new(self.clone())
    }
}

#[system]
#[read_component(DirLight)]
fn dir_light_impl(
    #[state] fill : &mut DirLightPipeline,
    world : &mut SubWorld,
    #[resource] encoder : &mut wgpu::CommandEncoder,
    #[resource] profiler : &mut GpuProfiler,
    #[resource] dst : &DirLightTexture,
    #[resource] gbuffer : &GFramebuffer
) {
    profiler.begin_scope("Dir light fill", encoder, &fill.render.device);
    let render = fill.render.clone();
    fill.draw(&render.device, encoder, world, &dst.tex, gbuffer);
    profiler.end_scope(encoder);
}

pub struct DirLightSystem {}

impl SchedulePlugin for DirLightSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("Directional light".into())
    }

    fn get_plugin_type(&self) -> PluginType {
        PluginType::Render
    }

    fn add_system(&self, game: &mut Game, builder: &mut Builder) {
        let pipeline = DirLightPipeline::new(&game.render_base, &game.scene.camera_buffer,
            wgpu::Extent3d {
                width : game.api.size.width,
                height : game.api.size.height,
                depth_or_array_layers : 1
            });

        builder.add_system(dir_light_impl_system(pipeline));
    }
}

pub struct DirLightPipeline {
    pub pipeline : wgpu::RenderPipeline,
    camera_bind_group_layout : wgpu::BindGroupLayout,
    light_bind_group_layout : wgpu::BindGroupLayout,
    camera_bind_group : wgpu::BindGroup,
    screen : ScreenMesh,
    light_groups : Vec<wgpu::BindGroup>,
    texture_bing_group_layout : wgpu::BindGroupLayout,
    diffuse : Option<wgpu::BindGroup>,
    normal : Option<wgpu::BindGroup>,
    position : Option<wgpu::BindGroup>,
    pub render : Arc<RenderBase>,
    size : wgpu::Extent3d
}

impl Pipeline for DirLightPipeline {

    fn new_described(desc: Box<dyn PipelineDesc>, camera_buffer: &Buffer) -> Self {
        let desc : Box<DirLightPipelineDesc> = desc.downcast().unwrap();
        DirLightPipeline::new(&desc.render, camera_buffer, desc.size)
    }

    fn get_desc(&self) -> Box<dyn PipelineDesc> {
        let mut desc = DirLightPipelineDesc {
            shader_path: AssetPath::GlobalPath("../../shaders/wgsl/point_light.wgsl".into()),
            render: self.render.clone(),
            size: self.size
        };
        Box::new(desc)
    }
}

impl DirLightPipeline {

    pub fn spawn_framebuffer(&self, device : &wgpu::Device, size : Extent3d) -> TextureBundle {
        TextureBundle::new(device, &wgpu::TextureDescriptor {
            label: Some("light buffer"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        }, wgpu::FilterMode::Nearest)
    }

    fn get_texture_layout(device : &wgpu::Device) -> wgpu::BindGroupLayout {
        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label : Some("Texture present binding"),
            entries : &[
                wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false 
                    },
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 1,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 2,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 3,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 4,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture { 
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2, 
                        multisampled: false 
                    },
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 5,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 6,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 7,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count : None
                },

            ]
        });
        texture_bind_group_layout
    }

    fn create_texture_group(&self, device : &wgpu::Device, src : &GFramebuffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &self.texture_bing_group_layout,
            entries : &[
                wgpu::BindGroupEntry {
                    binding : 0,
                    resource : wgpu::BindingResource::TextureView(&src.diffuse.view)
                },
                wgpu::BindGroupEntry {
                    binding : 1,
                    resource : wgpu::BindingResource::Sampler(&src.diffuse.sampler)
                },
                wgpu::BindGroupEntry {
                    binding : 2,
                    resource : wgpu::BindingResource::TextureView(&src.normal.view)
                },
                wgpu::BindGroupEntry {
                    binding : 3,
                    resource : wgpu::BindingResource::Sampler(&src.normal.sampler)
                },
                wgpu::BindGroupEntry {
                    binding : 4,
                    resource : wgpu::BindingResource::TextureView(&src.position.view)
                },
                wgpu::BindGroupEntry {
                    binding : 5,
                    resource : wgpu::BindingResource::Sampler(&src.position.sampler)
                },
                wgpu::BindGroupEntry {
                    binding : 6,
                    resource : wgpu::BindingResource::TextureView(&src.mr.view)
                },
                wgpu::BindGroupEntry {
                    binding : 7,
                    resource : wgpu::BindingResource::Sampler(&src.mr.sampler)
                },
            ],
            label : Some("texture present bind")
        })
    }

    pub fn new(render : &Arc<RenderBase>, camera_buffer : &wgpu::Buffer, size : wgpu::Extent3d) -> Self {
        let camera_bind_group_layout =
            render.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Canera uniform group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
        });

        let light_bind_group_layout =
            render.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Light uniform group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
        });

        let texture_bing_group_layout = DirLightPipeline::get_texture_layout(&render.device);

        let camera_bind_group = render.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &camera_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : camera_buffer.as_entire_binding()
            }],
            label : Some("camera bind group")
        });

        let shader = render.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/wgsl/dir_light.wgsl").into())
        });

        let pipeline_layout =
            render.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Test render layout"),
                bind_group_layouts : &[
                    &camera_bind_group_layout, 
                    &light_bind_group_layout, 
                    &texture_bing_group_layout,],
                push_constant_ranges: &[]
            });

        let pipeline = render.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers: &[space_core::SimpleVertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(wgpu::ColorTargetState {
                    format : wgpu::TextureFormat::Rgba32Float,
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


        Self {
            pipeline,
            camera_bind_group,
            camera_bind_group_layout,
            screen : TextureTransformPipeline::create_screen_mesh(&render.device),
            light_bind_group_layout,
            light_groups : vec![],
            texture_bing_group_layout,
            diffuse : None,
            normal : None,
            position : None,
            render : render.clone(),
            size,
        }
    }

    pub fn draw<'a>(
        &mut self, 
        device : &wgpu::Device, 
        encoder : &'a mut wgpu::CommandEncoder, 
        scene : &SubWorld,
        dst : &TextureBundle, 
        gbuffer : &GFramebuffer) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Dir light renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                view: &dst.view, 
                resolve_target: None, 
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }), 
                    store: true 
                }
            })],
            depth_stencil_attachment: None,
        });

        self.diffuse = Some(self.create_texture_group(device, &gbuffer));

        render_pass.set_bind_group(2, &self.diffuse.as_ref().unwrap(), &[]);

        self.light_groups.clear();
        let mut lights = <(&DirLight)>::query();
        for light in lights.iter(scene) {
            // let shadow = light.shadow.as_ref().unwrap();
            let light_uniform= device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("light"),
                layout: &self.light_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &light.buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });

            self.light_groups.push(light_uniform);
        }

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.screen.vertex.slice(..));
        // render_pass.set_index_buffer(self.sphere.index.slice(..), wgpu::IndexFormat::Uint32);
        for (idx, light) in lights.iter(scene).enumerate() {
            render_pass.set_bind_group(1, &self.light_groups[idx], &[]);
            render_pass.draw(0..6, 0..1);
        }
    }
}