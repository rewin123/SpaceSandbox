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
var t_ssao: texture_2d<f32>;
@group(0) @binding(1)
var s_ssao: sampler;

struct SmoothUniform {
    size : vec2<i32>
}

@group(0) @binding(2)
var<uniform> u : SmoothUniform;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    let step = vec2<f32>(1.0 / f32(u.size.x), 1.0 / f32(u.size.y));

    var res : f32 = 0.0;

    for (var dx = -1; dx < 2; dx++) {
        for (var dy = -1; dy < 2; dy++) {
            res += textureSample(t_ssao, s_ssao, in.uv + step * vec2<f32>(f32(dx), f32(dy))).r;
        }
    }

    res /= 9.0;
    res = pow(res, 2.0);
    out.diffuse = vec4<f32>(res, res, res, 1.0);

    return out;
}