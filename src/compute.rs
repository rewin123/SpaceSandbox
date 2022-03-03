use vulkano::buffer::*;
use vulkano::command_buffer::*;
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::descriptor_set::WriteDescriptorSet;
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::sync;
use vulkano::sync::GpuFuture;

#[test]
fn copy_buffer_test() {
    let mut rpu = crate::rpu::RPU::default();

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

#[test]
fn test_compute_shader() {
    let mut rpu = crate::rpu::RPU::default();

    let data_iter = 0..65536;
    let data_budffer = CpuAccessibleBuffer::from_iter(
        rpu.device.clone(),
        BufferUsage::all(),
        false,
        data_iter).expect("Failed to create buffer");

    let shader = cs::load(rpu.device.clone()).unwrap();

    let compute_pipeline = ComputePipeline::new(
        rpu.device.clone(),
        shader.entry_point("main").unwrap(),
        &(),
        None,
        |_| {},
    ).expect("Failed to create compte pipeline");

    let layout = compute_pipeline
        .layout()
        .descriptor_set_layouts()
        .get(0)
        .unwrap();

    let set = PersistentDescriptorSet::new(
        layout.clone(),
        [WriteDescriptorSet::buffer(0, data_budffer.clone())],
    ).unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        rpu.device.clone(),
        rpu.queue.family(),
        CommandBufferUsage::OneTimeSubmit
    ).unwrap();

    builder
        .bind_pipeline_compute(compute_pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Compute,
            compute_pipeline.layout().clone(),
            0,
            set
        ).dispatch([1024, 1, 1]).unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(rpu.device.clone())
        .then_execute(rpu.queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let content = data_budffer.read().unwrap();
    for (n, val) in content.iter().enumerate() {
        assert_eq!(*val, n as u32 * 12);
    }
    println!("Everything succeeded!");
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