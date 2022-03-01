
use engine::math::*;
use image::{ImageBuffer, Rgba};
use vulkano::{buffer::{CpuAccessibleBuffer, BufferUsage}, format::Format, image::view::ImageView, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents}, pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState}, GraphicsPipeline}, render_pass::Subpass, sync::{GpuFuture, self}};
use vulkano::render_pass::Framebuffer;

fn main() {
    let mut rpu = engine::rpu::RPU::default();

    let image = rpu.create_image(1024, 1024, Format::R8G8B8A8_UNORM).unwrap();
    let view = ImageView::new(image.clone()).unwrap();

    let buf = CpuAccessibleBuffer::from_iter(
        rpu.device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let vertex1 = Vec2 { position: [-0.5, -0.5] };
    let vertex2 = Vec2 { position: [ 0.0,  0.5] };
    let vertex3 = Vec2 { position: [ 0.5, -0.25] };

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        rpu.device.clone(),
        BufferUsage::all(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

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

    let framebuffer = Framebuffer::start(
        render_pass.clone())
        .add(view)
        .unwrap()
        .build()
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
            rpu.device.clone(),
            rpu.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        
    builder
        .begin_render_pass(
            framebuffer.clone(),
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .unwrap()
        .end_render_pass()
        .unwrap();

    let vs = vs::load(rpu.device.clone()).expect("Failed to load vertex shader");
    let fs = fs::load(rpu.device.clone()).expect("Failed to load fragment shader");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1024.0, 1024.0],
        depth_range: 0.0..1.0,
    };
    
    let pipeline = GraphicsPipeline::start()
        // Describes the layout of the vertex input and how should it behave
        .vertex_input_state(BuffersDefinition::new().vertex::<Vec2>())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // Indicate the type of the primitives (the default is a list of triangles)
        .input_assembly_state(InputAssemblyState::new())
        // Set the fixed viewport
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        // Same as the vertex input, but this for the fragment input
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        // This graphics pipeline object concerns the first pass of the render pass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that everything is specified, we call `build`.
        .build(rpu.device.clone())
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        rpu.device.clone(),
        rpu.queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();
    
    builder
        .begin_render_pass(
            framebuffer.clone(),
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .unwrap()
    
        // new stuff
        .bind_pipeline_graphics(pipeline.clone())
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .draw(
            3, 1, 0, 0, // 3 is the number of vertices, 1 is the number of instances
        )
        
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(image, buf.clone())
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(rpu.device.clone())
        .then_execute(rpu.queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
}


mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

layout(location = 0) out vec2 frag_pos;

void main() {
    frag_pos = position + vec2(1.0);
    gl_Position = vec4(position, 0.0, 1.0);
}"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

layout(location = 0) in vec2 frag_pos;

void main() {
    f_color = vec4(frag_pos.x, frag_pos.y, 1.0, 1.0);
}"
    }
}