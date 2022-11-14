use std::{num::NonZeroU32, sync::{Arc, Mutex}};
use std::collections::HashMap;
use wgpu::{Extent3d, Texture, TextureFormat};
use space_assets::*;
use space_core::{RenderBase, app::App};

use space_game::*;
use space_game::PluginName::Text;

use space_core::ecs::*;

#[derive(Resource)]
pub struct GFramebuffer {
    pub diffuse : TextureBundle,
    pub normal : TextureBundle,
    pub position : TextureBundle,
    pub mr : TextureBundle,
    pub depth : TextureBundle,
}

impl GFramebuffer {
    pub fn new(
        device : &wgpu::Device,
        size : wgpu::Extent3d) -> Self {

        let color_desc = wgpu::TextureDescriptor {
            label: Some("gbuffer color attachment"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        };

        let pos_desc = wgpu::TextureDescriptor {
            label: Some("gbuffer color attachment"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        };

        let noraml_desc = wgpu::TextureDescriptor {
            label: Some("gbuffer color attachment"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Snorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        };

        let diffuse = TextureBundle::new(device, &color_desc, wgpu::FilterMode::Nearest);
        let normal = TextureBundle::new(device, &noraml_desc, wgpu::FilterMode::Nearest);
        let position = TextureBundle::new(device, &pos_desc, wgpu::FilterMode::Nearest);
        let mr = TextureBundle::new(device, &color_desc, wgpu::FilterMode::Nearest);

        let depth = TextureBundle::new(device, &wgpu::TextureDescriptor {
            label: Some("gbuffer depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        }, wgpu::FilterMode::Nearest);

        Self {
            diffuse,
            normal,
            position,
            depth,
            mr
        }
    }

    pub fn spawn_renderpass<'a>(&'a self, encoder : &'a mut wgpu::CommandEncoder) -> wgpu::RenderPass {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Gbuffer render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.diffuse.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.normal.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.position.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }),
            Some(wgpu::RenderPassColorAttachment {
                view: &self.mr.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }),],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view : &self.depth.view,
                depth_ops: Some(wgpu::Operations {
                    load : wgpu::LoadOp::Clear(1.0),
                    store: true
                }),
                stencil_ops: None
            }),
        });

        render_pass
    }
}

#[derive(Resource)]
pub struct GBufferFill {
    pub pipeline : wgpu::RenderPipeline,
    camera_bind_group_layout : wgpu::BindGroupLayout,
    camera_bind_group : wgpu::BindGroup,
    texture_bind_group_layout : wgpu::BindGroupLayout,
    textures : HashMap<Material, Arc<wgpu::BindGroup>>,
    render : Arc<RenderBase>
}


fn gbuffer_filling(
    mut fill : ResMut<GBufferFill>,
    mut query : Query<(&GMeshPtr, &mut Material, &Location)>,
    mut gbuffer : ResMut<GFramebuffer>,
    mut assets : ResMut<SpaceAssetServer>,
    mut encoder : ResMut<RenderCommands>) {

    // profiler.begin_scope("GBuffer fill", encoder, &fill.render.device);
    fill.draw(query, gbuffer, assets, encoder);
    // profiler.end_scope(encoder);
}

pub struct GBufferPlugin {

}

impl SchedulePlugin for GBufferPlugin {
    fn get_name(&self) -> PluginName {
        PluginName::Text("GBiffer filling".into())
    }

    fn add_system(&self, app: &mut App) {
        let render = app.world.get_resource::<RenderApi>().unwrap().base.clone();
        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();

        let pipeline = GBufferFill::new(&render,
                         &app.world.get_resource::<CameraBuffer>().unwrap().buffer,
                         TextureFormat::Rgba32Float,
                         wgpu::Extent3d {
                             width : size.width,
                             height : size.height,
                             depth_or_array_layers : 1
                         });
        app.insert_resource(GBufferFill::spawn_framebuffer(&render.device, wgpu::Extent3d {
            width : size.width,
            height : size.height,
            depth_or_array_layers : 1
        }));
        app.insert_resource(pipeline);
        app.add_system_to_stage( GlobalStageStep::Render, gbuffer_filling);
    }
}

impl GBufferFill {


    pub fn spawn_framebuffer(device : &wgpu::Device, size : Extent3d) -> GFramebuffer {
        GFramebuffer::new(device, size)
    }

    pub fn new(render : &Arc<RenderBase>, camera_buffer : &wgpu::Buffer, format : wgpu::TextureFormat, size : wgpu::Extent3d) -> Self {
        let camera_bind_group_layout =
        render.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Canera uniform group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None
                        },
                        count: None
                    }
                ]
            });

        let camera_bind_group = render.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &camera_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : camera_buffer.as_entire_binding()
            }],
            label : Some("camera bind group")
        });

        let texture_bind_group_layout = render.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            ]
        });


        let shader = render.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/wgsl/gbuffer_fill.wgsl").into())
        });

        let pipeline_layout =
        render.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Test render layout"),
                bind_group_layouts : &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[]
            });

        let pipeline = render.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers: &GVertex::desc()
            },
            fragment: Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(wgpu::ColorTargetState {
                    format : wgpu::TextureFormat::Rgba8Unorm,
                    blend : None,
                    write_mask : wgpu::ColorWrites::ALL
                }),
                Some(wgpu::ColorTargetState {
                    format : wgpu::TextureFormat::Rgba8Snorm,
                    blend : None,
                    write_mask : wgpu::ColorWrites::ALL
                }),
                Some(wgpu::ColorTargetState {
                    format : wgpu::TextureFormat::Rgba32Float,
                    blend : None,
                    write_mask : wgpu::ColorWrites::ALL
                }),
                Some(wgpu::ColorTargetState {
                    format : wgpu::TextureFormat::Rgba8Unorm,
                    blend : None,
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
                depth_write_enabled: true,
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

        Self {
            pipeline,
            camera_bind_group,
            camera_bind_group_layout,
            textures : HashMap::new(),
            texture_bind_group_layout,
            render : render.clone()
        }
    }



    pub fn draw(&mut self,
                mut query : Query<(&GMeshPtr, &mut Material, &Location)>,
                mut gbuffer : ResMut<GFramebuffer>,
                mut assets : ResMut<SpaceAssetServer>,
                mut encoder : ResMut<RenderCommands>,) {

        let mut render_pass = gbuffer.spawn_renderpass(encoder.as_mut());

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

        for (mesh_ptr, mut material, loc) in &mut query {
            if material.need_rebind(assets.as_ref()) {
                let group = self.render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &self.texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&assets.get(&material.color).unwrap().view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&assets.get(&material.color).unwrap().sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&assets.get(&material.normal).unwrap().view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::Sampler(&assets.get(&material.normal).unwrap().sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: wgpu::BindingResource::TextureView(&assets.get(&material.metallic_roughness).unwrap().view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: wgpu::BindingResource::Sampler(&assets.get(&material.metallic_roughness).unwrap().sampler),
                        }
                    ],
                });

                material.gbuffer_bind = Some(group);
            }

            render_pass.set_bind_group(1, material.into_inner().gbuffer_bind.as_ref().unwrap(), &[]);
            render_pass.set_vertex_buffer(0, mesh_ptr.mesh.vertex.slice(..));
            render_pass.set_vertex_buffer(1, loc.buffer.slice(..));
            render_pass.set_index_buffer(mesh_ptr.mesh.index.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh_ptr.mesh.index_count, 0, 0..1);

        }
    }
}