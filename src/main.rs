
use vulkano::buffer::*;
use vulkano::command_buffer::*;
use vulkano::sync;
use vulkano::sync::GpuFuture;

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
    let data_budffer = CpuAccessibleBuffer::from_iter(
        rpu.device.clone(), 
        BufferUsage::all(),
    false,
        data_iter).expect("Failed to create buffer");

    
}

mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
    }
}