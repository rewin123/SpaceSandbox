use std::num::NonZeroU32;
use std::sync::Arc;
use bevy::prelude::{Handle, Assets, App};
use space_game::CameraBuffer;
use wgpu::{Buffer, Extent3d, TextureDimension};
use crate::light::PointLight;
use downcast_rs::*;
use space_assets::*;
use space_assets::wavefront::wgpu_load_gray_obj;
use space_core::ecs::{Query, World};
use space_core::RenderBase;
use crate::pipelines::{Pipeline, PipelineDesc};
use crate::pipelines::wgpu_gbuffer_fill::GFramebuffer;
use space_core::bevy::ecs::prelude::*;

#[derive(Resource)]
pub struct DirLightTexture {
    pub tex : TextureBundle
}

#[derive(Clone, Debug)]
pub struct PointLightPipelineDesc {
    shader_path : AssetPath,
    render : Arc<RenderBase>,
    size : wgpu::Extent3d
}

impl PipelineDesc for PointLightPipelineDesc {
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

#[derive(Resource)]
pub struct PointLightPipeline {
    pub pipeline : wgpu::RenderPipeline,
    camera_bind_group_layout : wgpu::BindGroupLayout,
    light_bind_group_layout : wgpu::BindGroupLayout,
    camera_bind_group : wgpu::BindGroup,
    sphere : Handle<GMesh>,
    light_groups : Vec<wgpu::BindGroup>,
    texture_bing_group_layout : wgpu::BindGroupLayout,
    diffuse : Option<wgpu::BindGroup>,
    normal : Option<wgpu::BindGroup>,
    position : Option<wgpu::BindGroup>,
    pub render : Arc<RenderBase>,
    size : wgpu::Extent3d,
    empty_shadow : wgpu::Texture,
    empty_shadow_view : wgpu::TextureView,
    empty_shadow_sample : wgpu::Sampler
}


impl PointLightPipeline {

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

    pub fn new(
            render : &Arc<RenderBase>, 
            size : wgpu::Extent3d,
            app : &mut App) -> Self {
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
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        count: None,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false
                        }
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None
                    }
                ]
        });

        let texture_bing_group_layout = PointLightPipeline::get_texture_layout(&render.device);

        let camera_bind_group = render.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &camera_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : app.world.get_resource::<CameraBuffer>().unwrap().buffer.as_entire_binding()
            }],
            label : Some("camera bind group")
        });

        let shader = render.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/wgsl/point_light.wgsl").into())
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
                buffers: &[GVertex::desc()[0].clone()]
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
                cull_mode : Some(wgpu::Face::Back),
                polygon_mode : wgpu::PolygonMode::Fill,
                unclipped_depth : false,
                conservative : false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format : wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default()
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None
        });

        let sphere = wgpu_load_gray_obj(
            &render.device, 
            "res/base_models/sphere.obj".into(),
            &mut app.world.get_resource_mut::<Assets<GMesh>>().unwrap()).unwrap();

        let tex = render.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: wgpu::Extent3d {
                width : 1,
                height : 1,
                depth_or_array_layers : 6
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        });

        let cube_view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("point light cube view"),
            format: None,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(NonZeroU32::new(6).unwrap())
        });

        let cube_sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow cube sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: Some(wgpu::CompareFunction::LessEqual),
            anisotropy_clamp: None,
            border_color: None
        });

        let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None
        });

        for idx in 0..6 {
            let view = tex.create_view(&wgpu::TextureViewDescriptor {
                label: Some("point light side view"),
                format: None,
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: Default::default(),
                base_mip_level: 0,
                mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                base_array_layer: idx,
                array_layer_count: Some(NonZeroU32::new(1).unwrap())
            });
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true
                    }),
                    stencil_ops: None
                })
            });
        }
        render.queue.submit(Some(encoder.finish()));

        Self {
            pipeline,
            camera_bind_group,
            camera_bind_group_layout,
            sphere : sphere[0].clone(),
            light_bind_group_layout,
            light_groups : vec![],
            texture_bing_group_layout,
            diffuse : None,
            normal : None,
            position : None,
            render : render.clone(),
            size,
            empty_shadow : tex,
            empty_shadow_view : cube_view,
            empty_shadow_sample : cube_sampler
        }
    }

    pub fn draw<'a>(
        &mut self, 
        device : &wgpu::Device, 
        encoder : &'a mut wgpu::CommandEncoder, 
        mut scene : Query<(&PointLight)>,
        dst : &TextureBundle, 
        gbuffer : &GFramebuffer,
        meshes : &mut Assets<GMesh>) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Point light renderpass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                view: &dst.view, 
                resolve_target: None, 
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }), 
                    store: true 
                }
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment { 
                view: &gbuffer.depth.view, 
                depth_ops: None, 
                stencil_ops: None
            }),
        });

        self.diffuse = Some(self.create_texture_group(device, &gbuffer));

        render_pass.set_bind_group(2, &self.diffuse.as_ref().unwrap(), &[]);

        self.light_groups.clear();
        for light in &mut scene {
            if let Some(shadow) = light.shadow.as_ref() {
                let light_uniform = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&shadow.cube_view)
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&shadow.sampler)
                        }
                    ],
                });
                self.light_groups.push(light_uniform);
            } else {
                let light_uniform = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&self.empty_shadow_view)
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&self.empty_shadow_sample)
                        }
                    ],
                });
                self.light_groups.push(light_uniform);
            }
        }
        let mesh = meshes.get(&self.sphere).unwrap();
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
        render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
        for (idx, light) in scene.iter().enumerate() {
            render_pass.set_bind_group(1, &self.light_groups[idx], &[]);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}