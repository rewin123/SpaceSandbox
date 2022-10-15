use std::num::NonZeroU32;
use egui::FontSelection::Default;
use wgpu::Extent3d;
use crate::{GMesh, GVertex, TextureView};

pub struct TextureBundle {
    pub texture : wgpu::Texture,
    pub view : wgpu::TextureView,
    pub sampler : wgpu::Sampler
}

impl TextureBundle {
    pub fn new(device : &wgpu::Device, desc : &wgpu::TextureDescriptor) -> Self {
        let texture = device.create_texture(desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None
        });
        Self {
            texture,
            view,
            sampler
        }
    }
}

pub struct GFramebuffer {
    pub diffuse : TextureBundle,
    pub normal : TextureBundle,
    pub position : TextureBundle,
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
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        };

        let diffuse = TextureBundle::new(device, &color_desc);

        let normal = TextureBundle::new(device, &color_desc);

        let normal = TextureBundle::new(device, &color_desc);

        let position = TextureBundle::new(device, &color_desc);

        let depth = TextureBundle::new(device, &wgpu::TextureDescriptor {
            label: Some("gbuffer depth"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING
        });


        Self {
            diffuse,
            normal,
            position,
            depth
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
                        a: 1.0,
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
                        a: 1.0,
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
                        a: 1.0,
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

pub struct GBufferFill {
    pub pipeline : wgpu::RenderPipeline,
    camera_bind_group_layout : wgpu::BindGroupLayout,
    camera_bind_group : wgpu::BindGroup,
}

impl GBufferFill {

    pub fn spawn_framebuffer(device : &wgpu::Device, size : Extent3d) -> GFramebuffer {
        GFramebuffer::new(device, size)
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