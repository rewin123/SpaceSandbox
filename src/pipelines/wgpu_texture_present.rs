use std::num::NonZeroU32;
use egui::FontSelection::Default;
use wgpu::{Extent3d, util::DeviceExt};
use crate::{GMesh, GVertex, TextureView};


pub struct TexturePresent {
    pub pipeline : wgpu::RenderPipeline,
    screen_mesh : ScreenMesh
}

impl TexturePresent {

    
    pub fn spawn_renderpass<'a>(&'a self, encoder : &'a mut wgpu::CommandEncoder, dst : &'a wgpu::TextureView) -> wgpu::RenderPass {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Texture present render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dst,
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
            depth_stencil_attachment: None,
        });

        render_pass
    }

    pub fn new(device : &wgpu::Device, format : wgpu::TextureFormat, size : wgpu::Extent3d) -> Self {

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/wgsl/present_texture.wgsl").into())
        });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label : Some("Texture present"),
                bind_group_layouts : &[],
                push_constant_ranges: &[]
            });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        Self {
            pipeline,
            screen_mesh : TexturePresent::create_screen_mesh(device)
        }
    }

    pub fn draw(&mut self, encoder : &mut wgpu::CommandEncoder, dst : &wgpu::TextureView) {
        let mut render_pass = self.spawn_renderpass(encoder, dst);
        render_pass.set_pipeline(&self.pipeline);
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