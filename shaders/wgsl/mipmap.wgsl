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
var t_tex: texture_2d<f32>;
@group(0) @binding(1)
var s_tex: sampler;

struct ScreenSize {
    width : f32,
    height : f32,
}

@group(0) @binding(2)
var<uniform> u : ScreenSize;


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    let step = vec2<f32>(1.0 / u.width, 1.0 / u.height);

    var res : vec4<f32> = textureSample();

    

    let pos = textureSample(t_position, s_position, in.uv).rgb;
    let normal = normalize(textureSample(t_normal, s_normal, in.uv).rgb);
    let rand_idx = get_random_idx(in.uv);
    let random_vec = ssao.random_vec[rand_idx].rgb;

    var tangent = normalize(random_vec - normal * dot(normal, random_vec));
    tangent = normalize(tangent - normal * dot(normal, ssao.random_vec[(rand_idx + 1) % 16].rgb));
    let bitangent = cross(tangent, normal);

    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    var res : f32 = 0.0;
    for (var idx = 0; idx < 32; idx++) {
        let np = pos + tbn * (ssao.samples[idx].rgb * ssao.scale);
        let clip = ssao.proj_view * vec4<f32>(np, 1.0);
        let uv = clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;
        let pos2 = textureSample(t_position, s_position, uv).rgb;
        // let clip2 = ssao.proj_view * vec4<f32>(pos2, 1.0);

        let dist = length(pos2 - pos);
        let k = smoothstep(0.0, 1.0, ssao.scale / dist);

        let s = f32((length(np - ssao.cam_pos.rgb)) > length(pos2 - ssao.cam_pos.rgb));

        res += s * k;
    }

    res /= 32.0;
    res = 1.0 - res;
    // res = pow(res, 2.0);
    out.ao = vec4<f32>(res, res, res, 1.0);

    return out;
}