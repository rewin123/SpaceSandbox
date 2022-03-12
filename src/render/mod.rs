
use std::sync::Arc;

use cgmath::*;
use specs::{Component, VecStorage, World, WorldExt, Join};
use vulkano::{device::Device, buffer::{CpuBufferPool, BufferUsage, cpu_pool::*, TypedBufferAccess}, memory::pool::StdMemoryPool, image::{view::ImageView, StorageImage, AttachmentImage}, format::{Format, ClearValue}, pipeline::{GraphicsPipeline, graphics::{vertex_input::BuffersDefinition, input_assembly::InputAssemblyState, viewport::{ViewportState, Viewport}, depth_stencil::DepthStencilState}, Pipeline, PipelineBindPoint}, render_pass::{Subpass, RenderPass, Framebuffer}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, sync::{self, GpuFuture}};
use crate::{mesh::{GpuMesh, Vertex}, rpu::RPU, game_object::Pos};

pub struct Camera {
    pub position : cgmath::Point3<f32>,
    pub forward : cgmath::Vector3<f32>,
    pub up : cgmath::Vector3<f32>,
    pub aspect_ratio : f32
}

pub struct GMesh {
    pub mesh: Arc<GpuMesh>
}

pub trait Render {

}

pub struct GRender {
    pub rpu : RPU,
    pub diffuse_img : Arc<StorageImage>,
    pub depth_img : Arc<AttachmentImage>,
    pub pipeline : Arc<GraphicsPipeline>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
}

impl GRender {
    pub fn from_rpu(rpu : RPU, w : u32, h : u32) -> Self {
        
        let diffuse_img = rpu.create_image(w, h, Format::R8G8B8A8_UNORM).unwrap();

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
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
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
        let depth_view = ImageView::new(self.depth_img.clone()).unwrap();

        let mut unifrom_buffer = camera.get_uniform_buffer(self.rpu.device.clone());
        let subbuffer = camera.get_subbuffer(&mut unifrom_buffer);

        let layout = self.pipeline.layout().descriptor_set_layouts().get(0).unwrap();

        let set = PersistentDescriptorSet::new(
            layout.clone(), [WriteDescriptorSet::buffer(0, subbuffer)]).unwrap();
        
        let framebuffer = Framebuffer::start(self.render_pass.clone())
            .add(diffuse_view).unwrap()
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
            vec![[0.0, 0.0, 0.0, 1.0].into(), 1f32.into()],
        ).unwrap()
        .set_viewport(0, [self.viewport.clone()])
        .bind_pipeline_graphics(self.pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline.layout().clone(),
            0,
            set.clone()
        );

        for (pos, gmesh) in (&read_pos, &read_mesh).join() {
            builder
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