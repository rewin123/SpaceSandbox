struct VertexInput {
    @location(0) position : vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    var out : VertexOutput;

    out.uv = vec2<f32>(model.position.x / 2.0 + 0.5, -model.position.y / 2.0 + 0.5);
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

struct FragmentOutput {
@location(0) ao : vec4<f32>,
};

@group(0) @binding(0)
var t_normal: texture_2d<f32>;
@group(0) @binding(1)
var s_normal: sampler;

@group(0) @binding(2)
var t_position: texture_2d<f32>;
@group(0) @binding(3)
var s_position: sampler;

struct SSAO {
    proj_view : mat4x4<f32>,
    samples : array<vec3<f32>, 32>
}

@group(0) @binding(4)
var<uniform> ssao : SSAO;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    return out;
}