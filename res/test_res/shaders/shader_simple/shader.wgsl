struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct Locals {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> r_locals: Locals;


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = tex_coord;
    var scale: f32 = 0.05;
    out.position = r_locals.transform * (position * vec4<f32>(scale, scale, scale, 1.0));
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var depth : f32 = in.position.z;
    depth = 1.0 - depth;
    return vec4<f32>(depth, depth, depth, 1.0);
}

