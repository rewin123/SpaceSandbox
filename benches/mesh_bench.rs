
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use SpaceSandbox::mesh::*;
use SpaceSandbox::mesh::wavefront::mesh_from_file;

pub fn small_wavefront_loading(c: &mut Criterion) {
    c.bench_function("wavefront",
    |b| b.iter(||  mesh_from_file( black_box(
        String::from("res/test_res/models/tomokitty/sculpt.obj")))));
}

criterion_group!(benches, small_wavefront_loading);
criterion_main!(benches);