use std::fs::rename;
use std::num::NonZeroU32;
use std::sync::Arc;
use bytemuck::{Zeroable, Pod};
use wgpu::{Extent3d, util::DeviceExt};
use space_assets::*;
use space_core::RenderBase;

pub trait TextureTransformUniform {
    fn get_bytes(&self) -> Vec<u8>;
}

pub struct CommonFramebuffer {
    pub dst : Vec<TextureBundle>
}

pub struct TextureTransformPipeline {
    pub pipeline : wgpu::RenderPipeline,
    screen_mesh : ScreenMesh,
    texture_bind_group_layout : wgpu::BindGroupLayout,
    output_format : wgpu::TextureFormat,
    output_count : u32,
    input_count : u32,
    size : Extent3d,
    render : Arc<RenderBase>,
    bind: Option<wgpu::BindGroup>,

    buffer : Option<wgpu::Buffer>
}

impl TextureTransformPipeline {

    pub fn spawn_framebuffer(&self) -> CommonFramebuffer {
        let mut textures = vec![];
        for idx in 0..self.output_count {
            textures.push(
                TextureBundle::new(&self.render.device, &wgpu::TextureDescriptor {
                    label: None,
                    size: self.size,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: self.output_format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
                })
            )
        }

        CommonFramebuffer {
            dst : textures
        }
    }
    
    pub fn spawn_renderpass<'a>(&'a self, encoder : &'a mut wgpu::CommandEncoder, dst : &'a [&TextureBundle]) -> wgpu::RenderPass {

        let mut attachs = vec![];
        for idx in 0..self.output_count {
            attachs.push(Some(wgpu::RenderPassColorAttachment {
                view: &dst[idx as usize].view,
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
            }))
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Texture present render Pass"),
            color_attachments: &attachs,
            depth_stencil_attachment: None,
        });

        render_pass
    }

    pub fn new(
        render : &Arc<RenderBase>,
        format : wgpu::TextureFormat,
        size : wgpu::Extent3d,
        input_count : u32,
        output_count : u32,
        uniform : Option<Box<dyn TextureTransformUniform>>,
        shader : String) -> Self {

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
        if let Some(u) = uniform.as_ref() {
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
        }
        

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

        let mut buffer = None;
        if let Some(s) = uniform {
            buffer = Some(render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &s.get_bytes(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE,
            }));
        }

        Self {
            pipeline,
            screen_mesh : TextureTransformPipeline::create_screen_mesh(&render.device),
            texture_bind_group_layout,
            output_format : format,
            output_count,
            input_count,
            size,
            render: render.clone(),
            bind: None,
            buffer
        }
    }


    pub fn draw(
            &mut self,
            device : &wgpu::Device,
            encoder : &mut wgpu::CommandEncoder,
            src : &[&TextureBundle],
            dst : &[&TextureBundle]) {

        let mut binds = vec![];
        for idx in 0..self.input_count {
            binds.push(
                wgpu::BindGroupEntry {
                    binding : idx * 2,
                    resource : wgpu::BindingResource::TextureView(&src[idx  as usize].view)
                },
            );
            binds.push(
                wgpu::BindGroupEntry {
                    binding : idx * 2 + 1,
                    resource : wgpu::BindingResource::Sampler(&src[idx as usize].sampler)
                }
            );
        }
        if let Some(buf) = self.buffer.as_ref() {
            binds.push(
                wgpu::BindGroupEntry {
                    binding: binds.len() as u32,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: buf,
                        offset: 0,
                        size: None,
                    }),
                }
            )
        }
        
        let tex_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
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