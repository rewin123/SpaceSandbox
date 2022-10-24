use std::iter;
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use space_core::{RenderBase, ScreenMesh, SimpleVertex};

pub struct MipmapGenerator {

}

impl MipmapGenerator {

    pub fn generate(
        render : &Arc<RenderBase>,
        tex : &wgpu::Texture,
        width : u32,
        height : u32,
        mipcount : u32,
        format : wgpu::TextureFormat) {

        let layout = render.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding : 0,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable : false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 1,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count : None
                },
                wgpu::BindGroupLayoutEntry {
                    binding : 2,
                    visibility : wgpu::ShaderStages::FRAGMENT,
                    ty : wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count : None
                }
            ]
        });

        let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let shader = render.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label : Some("Mipmap shader"),
            source : wgpu::ShaderSource::Wgsl(include_str!("../../../shaders/wgsl/mipmap.wgsl").into())
        });

        let pipeline_layout = render.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : None,
            bind_group_layouts : &[&layout],
            push_constant_ranges : &[]
        });

        let pipeline = render.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SimpleVertex::desc()]
            },
            fragment: Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                targets : &[Some(wgpu::ColorTargetState {
                    format,
                    blend : None,
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

        let mesh = MipmapGenerator::create_screen_mesh(&render.device);

        let mut encoder = render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Mipmap encoder")
        });

        let mut views = vec![];
        for mip in 0..mipcount {
            views.push(tex.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format : Some(format.clone()),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: mip,
                mip_level_count: NonZeroU32::new(1),
                base_array_layer: 0,
                array_layer_count: None
            }));
        }

        let mut uniforms = vec![];
        let mut binds = vec![];

        let mut mip_width = width;
        let mut mip_height = height;
        for mip in 1..mipcount {
            let buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[mip_width as f32, mip_height as f32]),
                usage: wgpu::BufferUsages::UNIFORM
            });

            let bind = render.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&views[mip as usize - 1])
                    },
                    wgpu::BindGroupEntry {
                        binding : 1,
                        resource : wgpu::BindingResource::Sampler(&sampler)
                    },
                    wgpu::BindGroupEntry {
                        binding : 2,
                        resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &buffer,
                            offset: 0,
                            size: None
                        })
                    }
                ]
            });

            binds.push(bind);
            uniforms.push(buffer);

            mip_width /= 2;
            mip_width = mip_width.max(1);

            mip_height /= 2;
            mip_height = mip_height.max(1);
        }

        for mip in 1..mipcount {
            let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mipmap renderpass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &views[mip as usize],
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
                    })
                ],
                depth_stencil_attachment: None
            });
            renderpass.set_pipeline(&pipeline);

            renderpass.set_bind_group(0, &binds[mip as usize - 1], &[]);
            renderpass.set_vertex_buffer(0, mesh.vertex.slice(..));
            renderpass.draw(0..6, 0..1);
        }

        render.queue.submit(iter::once(encoder.finish()));
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