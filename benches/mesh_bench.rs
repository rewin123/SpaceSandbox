
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use SpaceSandbox::mesh::*;
use SpaceSandbox::mesh::wavefront::mesh_from_file;

pub fn small_wavefront_loading(c: &mut Criterion) {
    c.bench_function("wavefront",
    |b| b.iter(||  mesh_from_file( black_box(
        String::from("res/test_res/models/tomokitty/sculpt.obj")))));
}

pub fn test_sponza_loading(c : &mut Criterion) {
    let rpu = SpaceSandbox::rpu::RPU::default();
    c.bench_function("sponza_easy_gltf_loading",
        |b| b.iter(|| SpaceSandbox::static_world::from_gltf(black_box("res/test_res/models/sponza/glTF/Sponza.gltf"), rpu.device.clone())));
}

criterion_group!(
    benches, 
    small_wavefront_loading, 
    test_sponza_loading);

criterion_main!(benches);