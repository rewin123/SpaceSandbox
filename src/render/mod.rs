
pub mod image_render;
pub mod shaders;

use std::sync::Arc;

use cgmath::*;
use specs::{Component, VecStorage, World, WorldExt, Join};
use vulkano::{device::{Device, Queue}, buffer::{CpuBufferPool, BufferUsage, cpu_pool::*, TypedBufferAccess, CpuAccessibleBuffer}, memory::pool::StdMemoryPool, image::{view::ImageView, StorageImage, AttachmentImage, MipmapsCount, ImmutableImage, ImageDimensions}, format::{Format, ClearValue}, pipeline::{GraphicsPipeline, graphics::{vertex_input::BuffersDefinition, input_assembly::InputAssemblyState, viewport::{ViewportState, Viewport}, depth_stencil::DepthStencilState}, Pipeline, PipelineBindPoint}, render_pass::{Subpass, RenderPass, Framebuffer}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, sync::{self, GpuFuture}, sampler::{Sampler, Filter, SamplerAddressMode, SamplerMipmapMode}};
use crate::{mesh::{GpuMesh, Vertex}, rpu::RPU, game_object::Pos};
use vulkano::image::*;
use vulkano::image::traits::*;

pub struct PointLight {
    pub position : cgmath::Point3<f32>,
    pub emissive : cgmath::Vector3<f32>,
}

pub struct Camera {
    pub position : cgmath::Point3<f32>,
    pub forward : cgmath::Vector3<f32>,
    pub up : cgmath::Vector3<f32>,
    pub aspect_ratio : f32
}

pub struct Material {
    pub base_color_factor : Vector4<f32>, 
    pub base_color_texture : Arc<ImmutableImage>,
}

pub struct GMesh {
    pub mesh: Arc<GpuMesh>,
    pub material : Arc<Material>
}

pub trait Render {

}

pub struct GView {
    pub diffuse_view : Arc<dyn vulkano::image::ImageViewAbstract>,
    pub normal_view : Arc<dyn vulkano::image::ImageViewAbstract>,
    pub pos_view : Arc<dyn vulkano::image::ImageViewAbstract>,
}

pub struct GRender {
    pub rpu : RPU,
    pub diffuse_img : Arc<StorageImage>,
    pub normal_img : Arc<StorageImage>,
    pub cam_pos_img : Arc<StorageImage>,
    pub depth_img : Arc<AttachmentImage>,
    pub pipeline : Arc<GraphicsPipeline>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
}

impl Material {
    pub fn from_gltf(
            mat : Arc<easy_gltf::Material>,
            rpu : RPU) -> Self {

        let diffuse_buf = mat.pbr.base_color_texture.clone().unwrap();

        let diffuse_dim = ImageDimensions::Dim2d {
            width: diffuse_buf.width(),
            height: diffuse_buf.height(),
            array_layers: 1,
        };

        let mut data = vec![];
        for p_data in diffuse_buf.iter() {
            data.push(*p_data);
        }

        let usage = ImageUsage {
            transfer_destination: true,
            transfer_source: true,
            sampled: true,
            ..ImageUsage::none()
        };
        let layout = ImageLayout::ShaderReadOnlyOptimal;

        let buffer = CpuAccessibleBuffer::from_iter(
            rpu.device.clone(),
            BufferUsage::transfer_source(),
            false,
            data,
        )
        .unwrap();

        let (diffuse_img, diffuse_future) = ImmutableImage::uninitialized(
            rpu.device.clone(),
            diffuse_dim,
            Format::R8G8B8A8_UNORM,
            diffuse_dim.max_mip_levels(),
            usage,
            vulkano::image::ImageCreateFlags::default(),
            layout,
            rpu.device.active_queue_families()
        ).unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            rpu.device.clone(),
            rpu.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();

        builder.copy_buffer_to_image_dimensions(
            buffer,
            diffuse_future,
            [0,0,0],
            diffuse_dim.width_height_depth(),
            0,
            1,
            0
        ).unwrap();

        for mipmap_dst in 1..diffuse_img.clone().mip_levels() {
            
            let src_width = diffuse_buf.width() / (2 as u32).pow(mipmap_dst - 1);
            let src_height = diffuse_buf.height() / (2 as u32).pow(mipmap_dst - 1);
            
            let dst_width = diffuse_buf.width() / (2 as u32).pow(mipmap_dst);
            let dst_height = diffuse_buf.height() / (2 as u32).pow(mipmap_dst);

            builder.blit_image(
                diffuse_img.clone(), 
                [0,0,0], 
                [src_width as i32 - 1, src_height as i32 - 1, 1], 
                0, 
                mipmap_dst - 1, 
                diffuse_img.clone(), 
                [0,0,0], 
                [dst_width as i32 - 1, dst_height as i32 - 1, 1], 
                0, 
                mipmap_dst, 
                1, 
                Filter::Linear).unwrap();
        }

        
        let command_buffer = builder.build().unwrap();

        let future = sync::now(rpu.device.clone())
            .then_execute(rpu.queue.clone(), command_buffer).unwrap()
            .then_signal_fence_and_flush().unwrap();

        future.wait(None).unwrap();
            

        Self {
            base_color_factor : mat.pbr.base_color_factor,
            base_color_texture: diffuse_img
        }
    }
}

impl GRender {

    pub fn get_gview(&self) -> GView {
        GView {
            diffuse_view : ImageView::new(self.diffuse_img.clone()).unwrap(),
            normal_view : ImageView::new(self.normal_img.clone()).unwrap(),
            pos_view : ImageView::new(self.cam_pos_img.clone()).unwrap(),
        }
    }

    pub fn from_rpu(rpu : RPU, w : u32, h : u32) -> Self {
        
        let diffuse_img = rpu.create_image(w, h, Format::R8G8B8A8_UNORM).unwrap();
        let normal_img = rpu.create_image(w, h, Format::R32G32B32A32_SFLOAT).unwrap();
        let cam_pos_img = rpu.create_image(w, h, Format::R32G32B32A32_SFLOAT).unwrap();

        let vs = standart_vertex::load(rpu.device.clone()).unwrap();
        let fs = gmesh_fragment::load(rpu.device.clone()).unwrap();

        let render_pass = vulkano::single_pass_renderpass!(rpu.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                },
                normal: {
                    load: Clear,
                    store: Store,
                    format: Format::R32G32B32A32_SFLOAT,
                    samples: 1,
                },
                cam_pos: {
                    load: Clear,
                    store: Store,
                    format: Format::R32G32B32A32_SFLOAT,
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color, normal, cam_pos],
                depth_stencil: {depth}
            }
        )
        .unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [w as f32, h as f32],
            depth_range: 0.0..1.0,
        };

        let pipeline = GraphicsPipeline::start()
            // Describes the layout of the vertex input and how should it behave
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
            // which one.
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            // Indicate the type of the primitives (the default is a list of triangles)
            .input_assembly_state(InputAssemblyState::new())
            // Set the fixed viewport
            .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport.clone()]))
            // Same as the vertex input, but this for the fragment input
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .depth_stencil_state(DepthStencilState::simple_depth_test())
            // This graphics pipeline object concerns the first pass of the render pass.
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            // Now that everything is specified, we call `build`.
            .build(rpu.device.clone())
            .unwrap();

        let depth_img = 
            AttachmentImage::transient(rpu.device.clone(), [w, h], Format::D16_UNORM).unwrap();

        Self {
            diffuse_img,
            normal_img,
            cam_pos_img,
            rpu : rpu.clone(), 
            pipeline,
            render_pass,
            viewport,
            depth_img,
        }
    }

    pub fn draw(&mut self, world : &World, camera : &Camera) {
        
        let read_mesh = world.read_storage::<GMesh>();
        let read_pos = world.read_storage::<Pos>();

        //clean image


        let diffuse_view = ImageView::new(self.diffuse_img.clone()).unwrap();
        let normal_view = ImageView::new(self.normal_img.clone()).unwrap();
        let cam_pos_view = ImageView::new(self.cam_pos_img.clone()).unwrap();
        let depth_view = ImageView::new(self.depth_img.clone()).unwrap();

        let mut unifrom_buffer = camera.get_uniform_buffer(self.rpu.device.clone());
        let subbuffer = camera.get_subbuffer(&mut unifrom_buffer);

        let layout = self.pipeline.layout().descriptor_set_layouts().get(0).unwrap();

        let set = PersistentDescriptorSet::new(
            layout.clone(), [WriteDescriptorSet::buffer(0, subbuffer)]).unwrap();
        
        let framebuffer = Framebuffer::start(self.render_pass.clone())
            .add(diffuse_view).unwrap()
            .add(normal_view).unwrap()
            .add(cam_pos_view).unwrap()
            .add(depth_view).unwrap()
            .build().unwrap();

        
        //do draw stuff
        let mut builder = AutoCommandBufferBuilder::primary(
            self.rpu.device.clone(),
            self.rpu.queue.family(),
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();
        
        builder
            .begin_render_pass(
                framebuffer.clone(),
                SubpassContents::Inline,
                vec![
                    [0.0, 0.0, 0.0, 1.0].into(), 
                    [0.0, 0.0, 0.0, 0.0].into(), 
                    [0.0, 0.0, 0.0, 0.0].into(),
                    1f32.into()],
            ).unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone());

        let sampler = Sampler::start(self.rpu.device.clone())
            .mag_filter(Filter::Linear)
            .min_filter(Filter::Linear)
            .address_mode(SamplerAddressMode::Repeat)
            .mipmap_mode(SamplerMipmapMode::Linear)
            .lod(0.0..=4.0)
            .build().unwrap();
    

        for (pos, gmesh) in (&read_pos, &read_mesh).join() {

            let base_tex_view = 
                ImageView::new(gmesh.material.base_color_texture.clone()).unwrap();

            let texture_set = PersistentDescriptorSet::new(
                self.pipeline.layout().descriptor_set_layouts().get(1).unwrap().clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    base_tex_view.clone(),
                    sampler.clone(),
                )],
            )
            .unwrap();

            builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                set.clone()
            )
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                1,
                texture_set.clone()
            )
            .bind_vertex_buffers(0, gmesh.mesh.verts.clone())
            .bind_index_buffer(gmesh.mesh.indices.clone())
            .draw_indexed(gmesh.mesh.indices.len() as u32, 1, 0, 0, 0).unwrap();
        }

        
        builder.end_render_pass().unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.rpu.device.clone())
            .then_execute(self.rpu.queue.clone(), command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).unwrap();

        
    }
}

impl Component for GMesh {
    type Storage = VecStorage<Self>;
}

impl Camera {

    pub fn get_right(&self) -> cgmath::Vector3<f32> {
        cgmath::Vector3::cross(self.forward, self.up).normalize()
    }

    pub fn rotate_camera(&mut self, dx : f32, dy : f32) {
        let right = self.get_right();
        self.forward = self.forward + dy * self.up;
        self.forward = cgmath::Vector3::normalize(self.forward);
        // self.up = right.cross(self.forward).normalize();
        let right = self.get_right();
        self.forward = self.forward + dx * right;
        self.forward = cgmath::Vector3::normalize(self.forward);
        
    }

    pub fn get_uniform_buffer(&self, device : Arc<Device>) -> CpuBufferPool<standart_vertex::ty::Data> {
        CpuBufferPool::<standart_vertex::ty::Data>::new(device.clone(), BufferUsage::all())
    }

    pub fn get_subbuffer(
        &self, 
        uniform_buffer : &mut CpuBufferPool<standart_vertex::ty::Data>)
            -> Arc<CpuBufferPoolSubbuffer<standart_vertex::ty::Data, Arc<StdMemoryPool>>> {
        let uniform_buffer_subbuffer = {

            let proj = cgmath::perspective(
                Rad(std::f32::consts::FRAC_PI_2),
                self.aspect_ratio,
                0.01,
                100.0,
            );
            let view = Matrix4::look_at_rh(
                self.position.clone(),
                self.position.clone() + self.forward.clone(),
                self.up.clone(),
            );
            let scale = Matrix4::from_scale(1.0);

            let uniform_data = standart_vertex::ty::Data {
                world: Matrix4::one().into(),
                view: (view * scale).into(),
                proj: proj.into(),
                cam_pos: self.position.clone().into(),
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        uniform_buffer_subbuffer
    }
}

pub mod standart_vertex {
    vulkano_shaders::shader!{
        ty: "vertex",
        path : "src/render/standart_vertex.glsl" ,
    }
}

pub mod gmesh_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/gmesh_fragment.glsl",
    }
}