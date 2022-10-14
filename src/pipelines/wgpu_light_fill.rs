use std::num::NonZeroU32;
use egui::FontSelection::Default;
use wgpu::{Extent3d, TextureDimension};
use crate::{GMesh, GVertex, TextureView};
use crate::wgpu_gbuffer_fill::TextureBundle;


pub struct PointLightPipeline {
    pub pipeline : wgpu::RenderPipeline,
    camera_bind_group_layout : wgpu::BindGroupLayout,
    camera_bind_group : wgpu::BindGroup,
}

impl PointLightPipeline {

    pub fn spawn_framebuffer(device : &wgpu::Device, size : Extent3d) -> TextureBundle {
        TextureBundle::new(device, &wgpu::TextureDescriptor {
            label: Some("light buffer"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        })
    }

    pub fn new(device : &wgpu::Device, camera_buffer : &wgpu::Buffer, format : wgpu::TextureFormat, size : wgpu::Extent3d) -> Self {
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout : &camera_bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : camera_buffer.as_entire_binding()
            }],
            label : Some("camera bind group")
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/wgsl/gbuffer_fill.wgsl").into())
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Test render layout"),
                bind_group_layouts : &[&camera_bind_group_layout],
                push_constant_ranges: &[]
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers: &[GVertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(wgpu::ColorTargetState {
                    format : wgpu::TextureFormat::Rgba32Float,
                    blend : None,
                    write_mask : wgpu::ColorWrites::ALL
                }),
                    Some(wgpu::ColorTargetState {
                        format : wgpu::TextureFormat::Rgba32Float,
                        blend : None,
                        write_mask : wgpu::ColorWrites::ALL
                    }),
                    Some(wgpu::ColorTargetState {
                        format : wgpu::TextureFormat::Rgba32Float,
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
        }
    }

    pub fn draw(&mut self, encoder : &mut wgpu::CommandEncoder, scene : &[GMesh], dst : &GFramebuffer) {
        let mut render_pass = dst.spawn_renderpass(encoder);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        for mesh in scene {
            render_pass.set_vertex_buffer(0, mesh.vertex.slice(..));
            render_pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
    }
}