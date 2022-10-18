struct CameraUniform {
    view : mat4x4<f32>,
    proj : mat4x4<f32>,
    pos : vec3<f32>
};

@group(0) @binding(0)
var<uniform> camera : CameraUniform;

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) tangent : vec3<f32>,
    @location(3) uv : vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) uv : vec2<f32>
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    var out : VertexOutput;
    out.normal = model.normal;
    out.pos = model.position;
    out.clip_position = camera.proj * camera.view * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    return out;
}

struct FragmentOutput {
@location(0) diffuse : vec4<f32>,
@location(1) normal : vec4<f32>,
@location(2) pos : vec4<f32>,
@location(3) mr : vec4<f32>,
};

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(2)
var t_normal: texture_2d<f32>;
@group(1) @binding(3)
var s_normal: sampler;

@group(1) @binding(4)
var t_mr: texture_2d<f32>;
@group(1) @binding(5)
var s_mr: sampler;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    out.diffuse = textureSample(t_diffuse, s_diffuse, in.uv);
    out.normal = vec4<f32>(in.normal, 1.0);
    out.pos = vec4<f32>(in.pos, 1.0);
    out.mr = textureSample(t_mr, s_mr, in.uv);

    return out;
}