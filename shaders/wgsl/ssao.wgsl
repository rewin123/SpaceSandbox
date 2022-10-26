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
    cam_pos : vec4<f32>,
    samples : array<vec4<f32>, 32>,
    random_vec : array<vec4<f32>, 16>,
    width : f32,
    height : f32,
    scale : f32
}

@group(0) @binding(4)
var<uniform> ssao : SSAO;

fn get_random_idx(uv : vec2<f32>) -> i32 {
    let scaled = uv * vec2<f32>(ssao.width, ssao.height);
    return (i32(scaled.y) % 4) * 4 + i32(scaled.x) % 4;
}

fn get_sample_random_idx(uv : vec2<f32>) -> i32 {
    let scaled = uv * vec2<f32>(ssao.width, ssao.height);
    return (i32(scaled.y * ssao.width) + i32(scaled.x)) % 32;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    let pos = textureSample(t_position, s_position, in.uv).rgb;
    let normal = normalize(textureSample(t_normal, s_normal, in.uv).rgb);
    let rand_idx = get_random_idx(in.uv);
    // let random_vec = ssao.random_vec[rand_idx].rgb;
    let random_vec = ssao.random_vec[0].rgb;
    let sample_vec_idx = get_sample_random_idx(in.uv);
    let sample_vec = ssao.samples[sample_vec_idx].rgb;

    var tangent = normalize(random_vec - normal * dot(normal, random_vec));
    // tangent = normalize(tangent - normal * dot(normal, ssao.random_vec[(rand_idx + 1) % 16].rgb));
    let bitangent = cross(tangent, normal);

    let tbn = mat3x3<f32>(tangent, bitangent, normal);

    var dt : f32 = 0.0;
    let t_step : f32 = ssao.scale / 32.0;
    let dir = tbn * sample_vec;

    var occlusion : f32 = 0.0;

    while (dt < ssao.scale) {
        let step = pos + dt * dir;
        let clip = ssao.proj_view * vec4<f32>(step, 1.0);
        let uv = clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;
        let pos2 = textureSample(t_position, s_position, uv);

        let dist2 = length(pos2.rgb - ssao.cam_pos.rgb);
        let dist = length(step - ssao.cam_pos.rgb);

        if (dist > dist2 + 0.01 && pos2.w != 0.0) {
            break;
        }

        dt = dt + t_step;
    }

    var res = smoothstep(0.0, ssao.scale, dt);

    // var res : f32 = 0.0;
    // for (var idx = 0; idx < 32; idx++) {
    //     let np = pos + tbn * (ssao.samples[idx].rgb * ssao.scale);
    //     let clip = ssao.proj_view * vec4<f32>(np, 1.0);
    //     let uv = clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;
    //     let pos2 = textureSample(t_position, s_position, uv);
    //     // let clip2 = ssao.proj_view * vec4<f32>(pos2, 1.0);

    //     let dp2 = pos2.rgb - pos;
    //     let dp1 = np - pos;

    //     let d2 = dot(dp2, normal);
    //     let d1 = dot(dp1, normal);

    //     let dist = length(pos2.rgb - pos);

    //     let s = f32(d2 > (d1 + 0.01));

    //     res += s * k * pos2.w;
    // }

    // res /= 32.0;
    // res = 1.0 - res;
     res = pow(res, 4.0);
    out.ao = vec4<f32>(res, res, res, 1.0);
//    out.ao = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    return out;
}