use std::borrow::BorrowMut;
use std::sync::Arc;
use std::time::Duration;
use SpaceSandbox::math::*;
use cgmath::Point3;
use image::{ImageBuffer, Rgba};
use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage, TypedBufferAccess}, image::view::ImageView, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState, depth_stencil::DepthStencilState}, GraphicsPipeline, Pipeline, PipelineBindPoint}, render_pass::Subpass, sync::{GpuFuture, self, FlushError}, swapchain::{self, AcquireError}, descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}};
use vulkano::render_pass::Framebuffer;
use winit::{event::{Event, WindowEvent}, event_loop::ControlFlow};
use SpaceSandbox::mesh::*;


fn main() {

    let (mut win_rpu, event_loop) = SpaceSandbox::rpu::WinRpu::default();
    let rpu = win_rpu.rpu.clone();

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(sync::now(rpu.device.clone()).boxed());

    let vs = SpaceSandbox::render::standart_vertex::load(rpu.device.clone()).unwrap();
    let fs = fs::load(rpu.device.clone()).unwrap();

    let mut cpu_mesh = mesh_from_file(
        String::from(r"C:\Users\rewin\OneDrive\Documents\GitHub\SpaceSandbox\res\test_res\models\tomokitty\sculpt.obj")).unwrap();

    cpu_mesh.scale(0.25 * 0.25 * 0.25);

    let mesh = GpuMesh::from_cpu(
        Arc::new(cpu_mesh),
        rpu.device.clone(),
        );

    let camera = SpaceSandbox::render::Camera {
        position: Point3::new(1.0, 1.0, 0.0),
        aspect_ratio : 1.0
    };

    let pipeline = GraphicsPipeline::start()
        // We need to indicate the layout of the vertices.
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        // See `vertex_shader`.
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        
        .depth_stencil_state(DepthStencilState::simple_depth_test())
        // We have to indicate which subpass of which render pass this pipeline is going to be used
        // in. The pipeline will only be usable from this particular subpass.
        .render_pass(Subpass::from(win_rpu.render_pass.clone(), 0).unwrap())
        // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
        .build(rpu.device.clone())
        .unwrap();

    let mut unifrom_buffer = camera.get_uniform_buffer(rpu.device.clone());
    

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            },
            Event::RedrawEventsCleared => {
                // It is important to call this function from time to time, otherwise resources will keep
                // accumulating and you will eventually reach an out of memory error.
                // Calling this function polls various fences in order to determine what the GPU has
                // already processed, and frees the resources that are no longer needed.
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                // Whenever the window resizes we need to recreate everything dependent on the window size.
                // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
                if recreate_swapchain {
                    
                }


                let subbuffer = camera.get_subbuffer(&mut unifrom_buffer);
                
                let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();

                let set = PersistentDescriptorSet::new(
                    layout.clone(),
                    [WriteDescriptorSet::buffer(0, subbuffer)]).unwrap();

                // let set = PersistentDescriptorSet::new(layout.clone(), pipeline
                //     [WriteDescriptorSet::buffer(0, uniform_buffer_subbuffer)]);
                
                // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
                // no image is available (which happens if you submit draw commands too quickly), then the
                // function will block.
                // This operation returns the index of the image that we are allowed to draw upon.
                //
                // This function can block if no image is available. The parameter is an optional timeout
                // after which the function call will return an error.
                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(win_rpu.swapchain.clone(), Some(Duration::from_millis(100))) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
                // will still work, but it may not display correctly. With some drivers this can be when
                // the window resizes, but it may not cause the swapchain to become out of date.
                if suboptimal {
                    recreate_swapchain = true;
                }

                // Specify the color to clear the framebuffer with i.e. blue
                let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into(), 1f32.into()];

                // In order to draw, we have to build a *command buffer*. The command buffer object holds
                // the list of commands that are going to be executed.
                //
                // Building a command buffer is an expensive operation (usually a few hundred
                // microseconds), but it is known to be a hot path in the driver and is expected to be
                // optimized.
                //
                // Note that we have to pass a queue family when we create the command buffer. The command
                // buffer will only be executable on that given queue family.
                let mut builder = AutoCommandBufferBuilder::primary(
                    rpu.device.clone(),
                    rpu.queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                builder
                    // Before we can draw, we have to *enter a render pass*. There are two methods to do
                    // this: `draw_inline` and `draw_secondary`. The latter is a bit more advanced and is
                    // not covered here.
                    //
                    // The third parameter builds the list of values to clear the attachments with. The API
                    // is similar to the list of attachments when building the framebuffers, except that
                    // only the attachments that use `load: Clear` appear in the list.
                    .begin_render_pass(
                        win_rpu.framebuffers[image_num].clone(),
                        SubpassContents::Inline,
                        clear_values,
                    )
                    .unwrap()
                    // We are now inside the first subpass of the render pass. We add a draw command.
                    //
                    // The last two parameters contain the list of resources to pass to the shaders.
                    // Since we used an `EmptyPipeline` object, the objects have to be `()`.
                    .set_viewport(0, [win_rpu.viewport.clone()])
                    .bind_pipeline_graphics(pipeline.clone())
                    .bind_descriptor_sets(
                        PipelineBindPoint::Graphics,
                        pipeline.layout().clone(),
                        0,
                        set.clone()
                    )
                    .bind_vertex_buffers(0, mesh.verts.clone())
                    .bind_index_buffer(mesh.indices.clone())
                    .draw_indexed(mesh.indices.len() as u32, 1, 0, 0, 0)
                    .unwrap()
                    // We leave the render pass by calling `draw_end`. Note that if we had multiple
                    // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
                    // next subpass.
                    .end_render_pass()
                    .unwrap();

                // Finish building the command buffer by calling `build`.
                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(rpu.queue.clone(), command_buffer)
                    .unwrap()
                    // The color output is now expected to contain our triangle. But in order to show it on
                    // the screen, we have to *present* the image by calling `present`.
                    //
                    // This function does not actually present the image immediately. Instead it submits a
                    // present command at the end of the queue. This means that it will only be presented once
                    // the GPU has finished executing the command buffer that draws the triangle.
                    .then_swapchain_present(rpu.queue.clone(), win_rpu.swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(rpu.device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(rpu.device.clone()).boxed());
                    }
                }
            },
            _ => ()
        }
    });
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

layout(location = 0) in vec3 v_normal;

void main() {
    f_color = vec4(v_normal.x, v_normal.y, v_normal.z, 1.0);
}"
    }
}