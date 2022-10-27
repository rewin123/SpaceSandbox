

struct AmbientLightUniform {
    color : vec3<f32>,
    cam_pos : vec3<f32>
}

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

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@group(0) @binding(4)
var t_position: texture_2d<f32>;
@group(0) @binding(5)
var s_position: sampler;

@group(0) @binding(6)
var t_mr: texture_2d<f32>;
@group(0) @binding(7)
var s_mr: sampler;

@group(0) @binding(8)
var t_ssao: texture_2d<f32>;
@group(0) @binding(9)
var s_ssao: sampler;

@group(0) @binding(10)
var<uniform> light : AmbientLightUniform;

struct FragmentOutput {
@location(0) color : vec4<f32>,
};


let PI = 3.14;


// From crate "bevy" and from https://www.unrealengine.com/en-US/blog/physically-based-shading-on-mobile
fn EnvBRDFApprox(f0: vec3<f32>, perceptual_roughness: f32, NoV: f32) -> vec3<f32> {
    let c0 = vec4<f32>(-1.0, -0.0275, -0.572, 0.022);
    let c1 = vec4<f32>(1.0, 0.0425, 1.04, -0.04);
    let r = perceptual_roughness * c0 + c1;
    let a004 = min(r.x * r.x, exp2(-9.28 * NoV)) * r.x + r.y;
    let AB = vec2<f32>(-1.04, 1.04) * a004 + r.zw;
    return f0 * AB.x + AB.y;
}


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;
    
    var screen_uv = in.uv;

    var N = textureSample(t_normal, s_diffuse, screen_uv).rgb;
    var pos = textureSample(t_position, s_position, screen_uv).rgb;
    var V = normalize(light.cam_pos - pos);

    var tex_color = textureSample(t_diffuse, s_diffuse, screen_uv).rgb;
    var mr = textureSample(t_mr, s_mr, screen_uv).rgb;

    var ssao = textureSample(t_ssao, s_ssao, screen_uv).r;

    var F0 = vec3<f32>(0.04,0.04,0.04);
    F0 = mix(F0, tex_color, mr.b);

    var NdotV = max(dot(N, V), 0.0);

    let diffuse = EnvBRDFApprox(tex_color, 1.0, NdotV);
    let specular = EnvBRDFApprox(F0, mr.g, NdotV);
    let Lo = (diffuse + specular) * light.color * ssao;
    out.color = vec4<f32>(Lo, 1.0);
    return out;
}