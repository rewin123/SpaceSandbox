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

@group(0) @binding(4)
var t_emissive: texture_2d<f32>;
@group(0) @binding(5)
var s_emissive: sampler;

@group(0) @binding(6)
var t_depth: texture_2d<f32>;
@group(0) @binding(7)
var s_depth: sampler;

@group(0) @binding(8)
var t_noise: texture_2d<f32>;
@group(0) @binding(9)
var s_noise: sampler;


struct SSDiffuse {
    proj_view : mat4x4<f32>,
    proj_view_inverse : mat4x4<f32>,
    random_vec : array<vec3<f32>, 64>,
    cam_pos : vec4<f32>,
    width : f32,
    height : f32,
    scale : f32
}

@group(0) @binding(10)
var<uniform> ssao : SSDiffuse;


@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;


    var sum = vec3<f32>(0.0, 0.0, 0.0);
    let start_pos = textureSample(t_position, s_position, in.uv).rgb;
    let start_color = textureSample(t_emissive, s_emissive, in.uv);
    let normal = textureSample(t_normal, s_normal, in.uv).rgb;
    let rv = textureSample(t_noise, s_noise, in.uv * vec2<f32>(ssao.width, ssao.height) / 128.0).rgb;
    let tangent = normalize(rv - normal * dot(rv, normal));
    let bitangent = cross(tangent, normal);
    let tbn = mat3x3(tangent, bitangent, normal);


    let cam_dist = length(start_pos - ssao.cam_pos.rgb);
    let range = 0.05 * cam_dist;

    var bounce = vec3<f32>(0.0, 0.0, 0.0);
    var ambient : f32 = 64.0;
    
    for (var i = 0; i < 64; i++) {
        var dir = tbn * ssao.random_vec[i] * ssao.scale;

        let step_pos = start_pos + dir;
        let clip = ssao.proj_view * vec4<f32>(step_pos, 1.0);
        let step_uv = clip.xy / clip.w * vec2<f32>(0.5, -0.5) + 0.5;
        let tex_dist = textureSample(t_depth, s_depth, step_uv).r;

        let step_dist = length(step_pos - ssao.cam_pos.rgb);
        let tex_pos = (step_pos - ssao.cam_pos.rgb) / step_dist * tex_dist + ssao.cam_pos.rgb;
        if (step_dist > (tex_dist + 0.01) && length(tex_pos - start_pos) <= ssao.scale) {
            ambient -= 1.0;
        }
    }

    ambient = ambient / 64.0;
    ambient = pow(ambient, 4.0);

    out.ao = vec4<f32>(vec3<f32>(ambient), start_color.w);
    return out;
}