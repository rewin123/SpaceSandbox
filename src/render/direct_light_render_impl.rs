use cgmath::*;
use specs::*;
use vulkano::{format::Format, pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState, depth_stencil::DepthStencilState}, GraphicsPipeline, PipelineBindPoint, Pipeline}, render_pass::{Subpass, Framebuffer}, image::{AttachmentImage, view::ImageView}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, sync::GpuFuture, buffer::{TypedBufferAccess, CpuBufferPool, BufferUsage}};
use vulkano::*;
use crate::{rpu::RPU, mesh::Vertex, game_object::{DirectLight, Pos}};

use super::{DirLightShadowRender, standart_vertex, shaders, Camera, GMesh};




impl DirLightShadowRender {
    pub fn from_rpu(rpu : RPU, w : u32, h : u32) -> Self {
        
        let vs = shaders::dir_light_shadow_vertex::load(rpu.device.clone()).unwrap();
        let fs = shaders::dir_light_shadow_fragment::load(rpu.device.clone()).unwrap();

        let render_pass = vulkano::single_pass_renderpass!(rpu.device.clone(),
            attachments: {
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
                color: [cam_pos],
                depth_stencil: {depth}
            }
        )
        .unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [w as f32, h as f32],
            depth_range: -1.0..1.0,
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
            rpu : rpu.clone(), 
            pipeline,
            render_pass,
            viewport,
        }
    }

    pub fn draw(&mut self, world : &World, camera : &Camera) {
        
        let read_dir_light = world.read_storage::<DirectLight>();
        
        for dir_light in (&read_dir_light).join() {

        match dir_light.textures.clone() {
            Some(texs) => {
                
                //clean image
                let cam_pos_view = ImageView::new(texs.pos_img.clone()).unwrap();
                let depth_view = ImageView::new(texs.depth_img.clone()).unwrap();

                let uniform_buffer = 
                    CpuBufferPool::<shaders::dir_light_shadow_vertex::ty::Data>::new(self.rpu.device.clone(), BufferUsage::all());

                let subbuffer = {
                    let forward = dir_light.dir;

                    let dz = {
                        if forward.z == 0.0 {
                            0.0
                        } else {
                            0.0
                        }
                    };

                    let mut up = Vector3::<f32>::new(
                        1.0, 
                        1.0, 
                        1.0
                    );

                    let mut right = up.cross(forward).normalize();
                    up = right.cross(forward).normalize();
        
                    let uniform_data = shaders::dir_light_shadow_vertex::ty::Data 
                    {
                        forward: forward.clone().into(),
                        up: up.clone().into(),
                        cam_pos: camera.position.clone().into(),
                        _dummy0 : [0, 0, 0, 0],
                        _dummy1 : [0, 0, 0, 0],
                    };
        
                    uniform_buffer.next(uniform_data).unwrap()
                };

                let layout = self.pipeline.layout().descriptor_set_layouts().get(0).unwrap();

                let set = PersistentDescriptorSet::new(
                    layout.clone(), [WriteDescriptorSet::buffer(0, subbuffer)]).unwrap();
                
                let framebuffer = Framebuffer::start(self.render_pass.clone())
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
                            [0.0, 0.0, 0.0, 0.0].into(),
                            1f32.into()],
                    ).unwrap()
                    .set_viewport(0, [self.viewport.clone()])
                    .bind_pipeline_graphics(self.pipeline.clone());

               

                    
                let read_gmesh = world.read_storage::<GMesh>();
                let read_pos = world.read_storage::<Pos>();

                for (pos, gmesh) in (&read_pos, &read_gmesh).join() {

                    builder
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            self.pipeline.layout().clone(),
                            0,
                            set.clone()
                        );

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
                None => {

                }
            }
        }
    }
}