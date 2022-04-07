use std::sync::Arc;

use specs::{World, WorldExt, Join};
use vulkano::{image::{view::ImageView, ImageViewAbstract, StorageImage}, pipeline::{GraphicsPipeline, graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState, depth_stencil::DepthStencilState}, Pipeline, PipelineBindPoint}, render_pass::{RenderPass, Subpass, Framebuffer}, format::Format, buffer::{CpuAccessibleBuffer, BufferUsage, TypedBufferAccess, CpuBufferPool}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, sampler::{Filter, Sampler, SamplerMipmapMode}, descriptor_set::PersistentDescriptorSet, sync::{self, GpuFuture}};
use vulkano::descriptor_set::*;

use crate::{rpu::RPU, game_object::DirectLight};

use super::{GView};

#[derive(Clone, Copy, Debug, Default)]
struct ImageVert {
    pub uv : [f32; 2]
}

impl ImageVert {
    pub fn new(x : f32, y : f32) -> Self {
        Self {
            uv : [x, y]
        }
    }
}

vulkano::impl_vertex!(ImageVert, uv);

pub struct DirectLightRender {
    pub rpu : RPU,
    pub target : Arc<dyn vulkano::image::view::ImageViewAbstract + Send + Sync>,
    pub pipeline : Arc<GraphicsPipeline>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
    square : Arc<CpuAccessibleBuffer<[ImageVert]>>
}

pub struct SimpleTextureRender {
    pub rpu : RPU,
    pub target : Arc<dyn vulkano::image::view::ImageViewAbstract + Send + Sync>,
    pub pipeline : Arc<GraphicsPipeline>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
    square : Arc<CpuAccessibleBuffer<[ImageVert]>>
}

pub struct MipmapBuffer {
    pub buffers : Vec<Arc<StorageImage>>
}

pub struct EyeImageRender {
    pub rpu : RPU,
    pub target : Arc<dyn vulkano::image::view::ImageViewAbstract + Send + Sync>,
    pub max_pipeline : Arc<GraphicsPipeline>,
    pub final_pipeline : Arc<GraphicsPipeline>,
    pub render_pass : Arc<RenderPass>,
    pub viewport : Viewport,
    square : Arc<CpuAccessibleBuffer<[ImageVert]>>,
    pub mipmap : Arc<MipmapBuffer>
}

impl MipmapBuffer {
    pub fn new(rpu : RPU, w : u32, h : u32, format : vulkano::format::Format) -> Arc<Self> {
        let mut res = Self { 
            buffers : vec![]
        };

        let mut wi = w;
        let mut hi = h;

        while wi > 1 || hi > 1 {

            let img = rpu.create_image(wi, hi, format).unwrap();
            res.buffers.push(img);
            //update sizes
            wi = wi / 2;
            hi = hi / 2;
            if wi == 0 {
                wi = 1;
            }
            if hi == 0 {
                hi = 1;
            }
        }


        Arc::new(res)
    }
}

impl DirectLightRender {
    pub fn new(
        rpu: RPU, 
        w : u32, 
        h : u32) -> Self {


        let square = CpuAccessibleBuffer::from_iter(
            rpu.device.clone(), 
            BufferUsage::all(),
            false,
            [
                ImageVert::new(0.0, 0.0),
                ImageVert::new(1.0, 0.0),
                ImageVert::new(0.0, 1.0),
                
                ImageVert::new(1.0, 1.0),
                ImageVert::new(1.0, 0.0),
                ImageVert::new(0.0, 1.0),
            ]
        ).unwrap();

        let target_img = rpu.create_image(w, h, Format::R8G8B8A8_UNORM).unwrap();

        let vs = image_vertex::load(rpu.device.clone()).unwrap();
        let fs = direct_light_fragment::load(rpu.device.clone()).unwrap();

        let render_pass = vulkano::single_pass_renderpass!(rpu.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [w as f32, h as f32],
            depth_range: 0.0..1.0,
        };

        let pipeline = GraphicsPipeline::start()
            // Describes the layout of the vertex input and how should it behave
            .vertex_input_state(BuffersDefinition::new().vertex::<ImageVert>())
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

        Self {
            rpu : rpu,
            pipeline,
            render_pass,
            viewport,
            target : ImageView::new(target_img).unwrap(),
            square
        }
    }

    pub fn draw(&self, world : &World, gview : GView) {

        let framebuffer = Framebuffer::start(self.render_pass.clone())
            .add(self.target.clone()).unwrap()
            .build().unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.rpu.device.clone(),
            self.rpu.queue.family(),
            CommandBufferUsage::MultipleSubmit,
        )
        .unwrap();

        let read_dir_light = world.read_storage::<DirectLight>();

        builder
            .begin_render_pass(
                framebuffer.clone(),
                SubpassContents::Inline,
                vec![
                    [0.0, 0.0, 0.0, 0.0].into(),]
            ).unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone());

        for (light,) in (&read_dir_light,).join() {
            let pool = 
            CpuBufferPool::<direct_light_fragment::ty::LightData>::new(
                self.rpu.device.clone(), BufferUsage::all());

        let subbuffer = {
            let uniform_data = direct_light_fragment::ty::LightData {
                direction : light.dir.into(),
                color : light.color.into(),
                _dummy0 : [0,0,0,0]
            };

            pool.next(uniform_data).unwrap()
        };

        let set = PersistentDescriptorSet::new(
            self.pipeline.layout().descriptor_set_layouts().get(1).unwrap().clone(), 
            [WriteDescriptorSet::buffer(0, subbuffer)]).unwrap();

        let sampler = Sampler::start(self.rpu.device.clone())
            .mag_filter(Filter::Linear)
            .min_filter(Filter::Linear)
            .mipmap_mode(SamplerMipmapMode::Linear)
            .build().unwrap();

        let texture_set = PersistentDescriptorSet::new(
                self.pipeline.layout().descriptor_set_layouts().get(0).unwrap().clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    gview.diffuse_view.clone(),
                    sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    gview.normal_view.clone(),
                    sampler.clone(),
                ),],
            )
            .unwrap();

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                texture_set.clone()
            )
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                1,
                set.clone()
            )
            .bind_vertex_buffers(0, self.square.clone())
            .draw(self.square.len() as u32, 1, 0, 0).unwrap();
        }

        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.rpu.device.clone())
            .then_execute(self.rpu.queue.clone(), command_buffer).unwrap()
            .then_signal_fence_and_flush().unwrap();

        future.wait(None).unwrap();
    }
}


impl SimpleTextureRender {
    pub fn new(
        rpu: RPU, 
        w : u32, 
        h : u32) -> Self {


        let square = CpuAccessibleBuffer::from_iter(
            rpu.device.clone(), 
            BufferUsage::all(),
            false,
            [
                ImageVert::new(0.0, 0.0),
                ImageVert::new(1.0, 0.0),
                ImageVert::new(0.0, 1.0),
                
                ImageVert::new(1.0, 1.0),
                ImageVert::new(1.0, 0.0),
                ImageVert::new(0.0, 1.0),
            ]
        ).unwrap();

        let target_img = rpu.create_image(w, h, Format::R8G8B8A8_UNORM).unwrap();

        let vs = image_vertex::load(rpu.device.clone()).unwrap();
        let fs = image_fragment::load(rpu.device.clone()).unwrap();

        let render_pass = vulkano::single_pass_renderpass!(rpu.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [w as f32, h as f32],
            depth_range: 0.0..1.0,
        };

        let pipeline = GraphicsPipeline::start()
            // Describes the layout of the vertex input and how should it behave
            .vertex_input_state(BuffersDefinition::new().vertex::<ImageVert>())
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

        Self {
            rpu : rpu,
            pipeline,
            render_pass,
            viewport,
            target : ImageView::new(target_img).unwrap(),
            square
        }
    }

    pub fn draw(&self, gview : GView) {

        let framebuffer = Framebuffer::start(self.render_pass.clone())
            .add(self.target.clone()).unwrap()
            .build().unwrap();

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
                    [0.0, 0.0, 0.0, 0.0].into(),]
            ).unwrap()
            .set_viewport(0, [self.viewport.clone()])
            .bind_pipeline_graphics(self.pipeline.clone());

        let sampler = Sampler::start(self.rpu.device.clone())
            .mag_filter(Filter::Linear)
            .min_filter(Filter::Linear)
            .mipmap_mode(SamplerMipmapMode::Linear)
            .build().unwrap();

        let texture_set = PersistentDescriptorSet::new(
                self.pipeline.layout().descriptor_set_layouts().get(0).unwrap().clone(),
                [WriteDescriptorSet::image_view_sampler(
                    0,
                    gview.diffuse_view.clone(),
                    sampler.clone(),
                ),
                WriteDescriptorSet::image_view_sampler(
                    1,
                    gview.normal_view.clone(),
                    sampler.clone(),
                ),],
            )
            .unwrap();

        builder
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                texture_set.clone()
            )
            .bind_vertex_buffers(0, self.square.clone())
            .draw(self.square.len() as u32, 1, 0, 0).unwrap();

        builder.end_render_pass().unwrap();

        let command_buffer = builder.build().unwrap();

        let future = sync::now(self.rpu.device.clone())
            .then_execute(self.rpu.queue.clone(), command_buffer).unwrap()
            .then_signal_fence_and_flush().unwrap();

        future.wait(None).unwrap();
    }
}


impl EyeImageRender {
    pub fn new(
        rpu: RPU, 
        w : u32, 
        h : u32) -> Self {


        let square = CpuAccessibleBuffer::from_iter(
            rpu.device.clone(), 
            BufferUsage::all(),
            false,
            [
                ImageVert::new(0.0, 0.0),
                ImageVert::new(1.0, 0.0),
                ImageVert::new(0.0, 1.0),
                
                ImageVert::new(1.0, 1.0),
                ImageVert::new(1.0, 0.0),
                ImageVert::new(0.0, 1.0),
            ]
        ).unwrap();

        let target_img = rpu.create_image(w, h, Format::R8G8B8A8_UNORM).unwrap();


        let render_pass = vulkano::single_pass_renderpass!(rpu.device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap();

        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [w as f32, h as f32],
            depth_range: 0.0..1.0,
        };
        
        let max_pipeline = {
            let vs = image_vertex::load(rpu.device.clone()).unwrap();
            let fs = crate::render::shaders::max_image_fragment::load(rpu.device.clone()).unwrap();
    
            let pipeline = GraphicsPipeline::start()
                // Describes the layout of the vertex input and how should it behave
                .vertex_input_state(BuffersDefinition::new().vertex::<ImageVert>())
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

            pipeline
        };

        let final_pipeline = {
            let vs = image_vertex::load(rpu.device.clone()).unwrap();
            let fs = crate::render::shaders::eye_fragment::load(rpu.device.clone()).unwrap();
    
            let pipeline = GraphicsPipeline::start()
                // Describes the layout of the vertex input and how should it behave
                .vertex_input_state(BuffersDefinition::new().vertex::<ImageVert>())
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

            pipeline
        };

        

        Self {
            rpu : rpu.clone(),
            max_pipeline,
            final_pipeline,
            render_pass,
            viewport,
            target : ImageView::new(target_img).unwrap(),
            square,
            mipmap : MipmapBuffer::new(rpu.clone(), w, h, vulkano::format::Format::R32G32B32A32_SFLOAT)
        }
    }

    fn calc_max(&mut self, view : Arc<dyn ImageViewAbstract>) {

    }

    pub fn draw(&self, gview : GView) {

        let framebuffer = Framebuffer::start(self.render_pass.clone())
            .add(self.target.clone()).unwrap()
            .build().unwrap();

        
    }
}

pub mod image_vertex {
    vulkano_shaders::shader!{
        ty: "vertex",
        path : "src/render/image_vert.glsl",
    }
}

pub mod image_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/image_frag.glsl",
    }
}

pub mod direct_light_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/direct_light_frag.glsl",
    }
}