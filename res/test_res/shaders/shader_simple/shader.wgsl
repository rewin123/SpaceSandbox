struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct Locals {
    pos: vec4<f32>;
    frw: vec4<f32>;
    up: vec4<f32>;
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
    var local_pos : vec4<f32> = position * vec4<f32>(scale, scale, scale, 1.0);
    local_pos = local_pos - r_locals.pos;

    var cam_z = r_locals.frw.x * local_pos.x + r_locals.frw.y * local_pos.y + r_locals.frw.z * local_pos.z;
    var cam_y = r_locals.up.x * local_pos.x + r_locals.up.y * local_pos.y + r_locals.up.z * local_pos.z;
    // var cam_x = r_locals.up.x * local_pos.x + r_locals.up.y * local_pos.y + r_locals.up.z * local_pos.z;

    var right = cross(r_locals.frw.xyz, r_locals.up.xyz);
    var cam_x = dot(local_pos.xyz, right);

    out.position = vec4<f32>(cam_x / cam_z, cam_y / cam_z, (cam_z - 0.1) * 0.1, 1.0);
    out.position = out.position;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var depth : f32 = in.position.z;
    depth = 1.0 - depth;
    return vec4<f32>(depth, depth, depth, 1.0);
}

