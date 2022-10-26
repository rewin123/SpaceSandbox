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
@location(0) diffuse : vec4<f32>,
};

@group(0) @binding(0)
var t_pos: texture_2d<f32>;
@group(0) @binding(1)
var s_pos: sampler;
@group(0) @binding(2)
var<uniform> cam_pos : vec4<f32>;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    let pos = textureSample(t_pos, s_pos, in.uv);
    let res = length(pos.rgb - cam_pos.rgb) + 10000.0 * (1.0 - pos.w);

    out.diffuse = vec4<f32>(res, res, res, 1.0);

    return out;
}