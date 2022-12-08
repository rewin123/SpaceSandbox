use std::num::NonZeroU32;
use std::sync::Arc;
use bevy::prelude::{Handle, Assets};
use wgpu::{Extent3d, TextureDimension};
use crate::light::{PointLight, PointLightShadow};
use space_shaders::*;
use space_core::RenderBase;
use space_assets::*;
use space_core::ecs::*;

#[derive(Resource)]
pub struct PointLightShadowPipeline {
    pub pipeline : wgpu::RenderPipeline,
    light_part_layout : wgpu::BindGroupLayout,
    pub render : Arc<RenderBase>
}

impl PointLightShadowPipeline {

    pub fn new(render : &Arc<RenderBase>) -> Self {
        let light_part_layout =
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

        let shader = render.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../../../shaders/wgsl/light_camera_shadow.wgsl").into())
        });

        let pipeline_layout =
            render.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Test render layout"),
                bind_group_layouts : &[
                    &light_part_layout,
                ],
                push_constant_ranges: &[]
            });

        let pipeline = render.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Shadow pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers: &GVertex::desc()
            },
            fragment: Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[]
            }),
            primitive: wgpu::PrimitiveState {
                topology : wgpu::PrimitiveTopology::TriangleList,
                strip_index_format : None,
                front_face : wgpu::FrontFace::Ccw,
                cull_mode : Some(wgpu::Face::Front),
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
            light_part_layout,
            render : render.clone()
        }
    }


    pub fn draw<'a>(
        &mut self,
        encoder : &'a mut wgpu::CommandEncoder,
        mut mesh_query : Query<(&Handle<GMesh>, &Material, &TransformBuffer)>,
        mut light_query : Query<(&mut PointLight)>,
        mut meshes : ResMut<Assets<GMesh>>) {

        for mut light in light_query.iter_mut() {
            if let Some(shadow) = light.shadow.as_mut() {
                if shadow.camera_binds.len() == 0 {
                    //create bind group
                    for idx in 0..6 {
                        let bind = self.render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Light camera bind to shadow mapping"),
                            layout: &self.light_part_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                        buffer: &shadow.camera_buffers[idx],
                                        offset: 0,
                                        size: None
                                    })
                                }
                            ]
                        });
                        shadow.camera_binds.push(bind);
                    }
                }
            }
        }

        for light in light_query.iter() {
            if let Some(shadow) = light.shadow.as_ref() {
                for camera_idx in 0..6 {
                    // profiler.begin_scope("Shadow pass", encoder, &self.render.device);
                    self.shadow_draw(shadow, camera_idx, &mut mesh_query, encoder, &mut meshes);
                    // profiler.end_scope(encoder);
                }
            }
        }
    }


    fn shadow_draw(&mut self,
                   shadow : &PointLightShadow,
                   idx : usize,
                   query : &mut Query<(&Handle<GMesh>, &Material, &TransformBuffer)>,
                   encoder : &mut wgpu::CommandEncoder,
                   meshes : &mut ResMut<Assets<GMesh>>) {

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Point light renderpass"),
            color_attachments: &[],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &shadow.side_views[idx],
                depth_ops: Some(wgpu::Operations {
                    load : wgpu::LoadOp::Clear(1.0),
                    store: true
                }),
                stencil_ops: None
            }),
        });
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &shadow.camera_binds[idx], &[]);

        for (mesh_ptr, material, loc) in query.iter() {
            let mesh = meshes.get(mesh_ptr).unwrap();

            render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
            render_pass.set_vertex_buffer(1, loc.buffer.slice(..));
            render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}