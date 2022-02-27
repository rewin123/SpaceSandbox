
use image::ImageBuffer;
use image::Rgba;
use vulkano::buffer::*;
use vulkano::command_buffer::*;
use vulkano::format::ClearValue;
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano::pipeline::*;
use vulkano::descriptor_set::*;

#[derive(Clone, Copy)]
struct Vertex {
    x : f32,
    y : f32,
    z : f32
}

#[test]
fn copy_buffer_test() {
    let mut rpu = engine::rpu::RPU::default();

    let source_content = 0..64;
    let source = CpuAccessibleBuffer::from_iter(rpu.device.clone(), BufferUsage::all(), false, source_content)
        .expect("failed to create buffer");

    let destination_content = (0..64).map(|_| 0);
    let destination = CpuAccessibleBuffer::from_iter(rpu.device.clone(), BufferUsage::all(), false, destination_content)
        .expect("failed to create buffer");

    let mut builder = AutoCommandBufferBuilder::primary(
        rpu.device.clone(),
        rpu.queue.family(),
        CommandBufferUsage::OneTimeSubmit
    ).unwrap();

    builder.copy_buffer(source.clone(), destination.clone()).unwrap();

    let command_buffer = builder.build().expect("Fail to build command buffer");

    let future = sync::now(rpu.device.clone())
        .then_execute(rpu.queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let src_content = source.read().unwrap();
    let destination_content = destination.read().unwrap();
    assert_eq!(&*src_content, &*destination_content);
}

fn main() {
    let mut rpu = engine::rpu::RPU::default();

    let data_iter = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(
        rpu.device.clone(), 
        BufferUsage::all(),
    false,
        data_iter).expect("Failed to create buffer");

    let shader = cs::load(rpu.device.clone())
        .expect("failed to create shader module");

    let image = rpu.create_image(
        1024, 
        1024, 
        vulkano::format::Format::R8G8B8A8_UNORM).unwrap();

    let buf = CpuAccessibleBuffer::from_iter(
        rpu.device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("failed to create buffer");

    let compute_pipeline = ComputePipeline::new(
            rpu.device.clone(),
            shader.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        ).expect("failed to create compute pipeline");

    let layout = compute_pipeline
        .layout()
        .descriptor_set_layouts()
        .get(0)
        .unwrap();

    let set = PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, data_buffer.clone())], // 0 is the binding
        )
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
            rpu.device.clone(),
            rpu.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

    builder
        .clear_color_image(image.clone(), ClearValue::Float([0.0, 0.0, 1.0, 1.0]))
        .unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone()) // new
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

mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: "
        #version 450

        layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
        
        layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
        
        void main() {
            vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
            vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
        
            vec2 z = vec2(0.0, 0.0);
            float i;
            for (i = 0.0; i < 1.0; i += 0.005) {
                z = vec2(
                    z.x * z.x - z.y * z.y + c.x,
                    z.y * z.x + z.x * z.y + c.y
                );
        
                if (length(z) > 4.0) {
                    break;
                }
            }
        
            vec4 to_write = vec4(vec3(i), 1.0);
            imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
        }"
    }
}