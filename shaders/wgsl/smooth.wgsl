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

@group(0) @binding(2)
var t_depth: texture_2d<f32>;
@group(0) @binding(3)
var s_depth: sampler;

struct SmoothUniform {
    size : vec2<f32>
}

@group(0) @binding(4)
var<uniform> u : SmoothUniform;

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    let step = vec2<f32>(1.0 / u.size.x, 1.0 / u.size.y) * 2.0;

    var res : f32 = 0.0;
    var weight_sum : f32 = 0.0;
    let center_depth = textureSample(t_depth, s_depth, in.uv).r;
    let center_ssao = textureSample(t_ssao, s_ssao, in.uv).r;


    for (var dx = -2; dx < 3; dx++) {
        for (var dy = -2; dy < 3; dy++) {
            let dist = f32(dx * dx + dy * dy);
            let duv = in.uv + step * vec2<f32>(f32(dx), f32(dy));
            let depth = textureSample(t_depth, s_depth, duv).r;
            let pix_ssao = textureSample(t_ssao, s_ssao, duv).r;
            let k = exp(-(depth - center_depth) * (depth - center_depth))
                * exp(-(pix_ssao - center_ssao) * (pix_ssao - center_ssao));
            res += pix_ssao * k;
            weight_sum += k;
        }
    }

    res /= weight_sum;
//    res = textureSample(t_ssao, s_ssao, in.uv).r;
//    res = pow(res, 4.0);
    out.diffuse = vec4<f32>(res, res, res, 1.0);

    return out;
}