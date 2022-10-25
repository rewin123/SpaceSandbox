use std::fs::rename;
use std::num::NonZeroU32;
use std::sync::Arc;
use bytemuck::Zeroable;
use rand::Rng;
use wgpu::{Extent3d, util::DeviceExt};
use space_assets::*;
use space_core::RenderBase;
use crate::{pipelines::{CommonFramebuffer, GFramebuffer}, Camera};
use encase::*;

#[repr(C)]
#[derive(Zeroable, bytemuck::Pod, Clone, Copy)]
struct SsaoUniform {
    proj_view : [[f32; 4]; 4],
    cam_pos : [f32; 4],
    samples : [[f32; 4]; 32],
    random_vec  : [[f32; 4]; 16],
    tex_width : f32,
    tex_height : f32,
    scale : f32,
    dummy1 : f32
}

impl Default for SsaoUniform {
    fn default() -> Self {
        Self {
            proj_view : [[0.0; 4]; 4],
            cam_pos : [0.0; 4],
            samples : [[0.0; 4] ; 32],
            random_vec : [[0.0; 4]; 16],
            tex_width : 0.0,
            tex_height : 0.0,
            scale : 0.0,
            dummy1 : 0.0
        }
    }
}

pub struct SSAO {
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
    cached_uniform : SsaoUniform
}

impl SSAO {

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
                }, wgpu::FilterMode::Nearest)
            )
        }

        CommonFramebuffer {
            dst : textures
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

    pub fn new(
        render : &Arc<RenderBase>,
        format : wgpu::TextureFormat,
        size : wgpu::Extent3d,
        input_count : u32,
        output_count : u32,
        shader : String) -> Self {

        let input_count = 2;
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
            binding: 4,
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

        let mut def_uniform = SsaoUniform::default();

        let mut samples = [[0.0f32; 4]; 32];
        let mut thread_rng = rand::thread_rng();
        for i in 0..32 {
            let mut v = nalgebra::Vector3::new(
                thread_rng.gen_range(-1.0f32..=1.0),
                thread_rng.gen_range(-1.0f32..=1.0),
                thread_rng.gen_range(0.1f32..=1.0)
            );
            
            v = v.normalize();
            let mut scale = (i as f32) / 32.0;
            scale = SSAO::lerp(scale * scale, 0.1, 1.0);
            v = v * scale;
            samples[i] = [v.x, v.y, v.z, 1.0];
        }

        let mut random_vec = [[0.0f32; 4]; 16];
        for i in 0..16 {
            let mut v = nalgebra::Vector3::new(
                thread_rng.gen_range(-1.0..=1.0),
                thread_rng.gen_range(-1.0..=1.0),
                thread_rng.gen_range(-1.0..=1.0)
            );
            v = v.normalize();
            random_vec[i] = [v.x, v.y, v.z, 1.0];
        }

        def_uniform.random_vec = random_vec;
        def_uniform.samples = samples;


        let buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&def_uniform),
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::UNIFORM,
        });

        Self {
            pipeline,
            screen_mesh : SSAO::create_screen_mesh(&render.device),
            texture_bind_group_layout,
            output_format : format,
            output_count,
            input_count,
            size,
            render: render.clone(),
            bind: None,
            buffer : Arc::new(buffer),
            scale : 1.0,
            cached_uniform : def_uniform
        }
    }

    pub fn update(&mut self, camera : &Camera) {
        let cam_uniform = camera.build_uniform();

        let proj_view = cam_uniform.proj * cam_uniform.view;

        self.cached_uniform.proj_view = proj_view.into();
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