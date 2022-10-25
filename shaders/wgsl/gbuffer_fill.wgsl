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
    @location(3) uv : vec2<f32>,
    @location(4) model_mat_1 : vec4<f32>,
    @location(5) model_mat_2 : vec4<f32>,
    @location(6) model_mat_3 : vec4<f32>,
    @location(7) model_mat_4 : vec4<f32>,
}


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) pos: vec3<f32>,
    @location(2) uv : vec2<f32>,
    @location(3) tangent : vec3<f32>
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    let model_mat = mat4x4<f32>(model.model_mat_1,model.model_mat_2,model.model_mat_3,model.model_mat_4);
    var out : VertexOutput;
    out.normal = model.normal;
    out.pos = (model_mat * vec4<f32>(model.position, 1.0)).rgb;
    out.clip_position = camera.proj * camera.view * model_mat * vec4<f32>(model.position, 1.0);
    out.uv = model.uv;
    out.tangent = model.tangent;
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


fn normal_mapping(normal : vec3<f32>, tangent : vec3<f32>, uv : vec2<f32>) -> vec4<f32> {
    let bitangent = -cross(normal, tangent);

    var map = textureSample(t_normal, s_normal, uv).rgb;
    map = map * 2.0 - 1.0;

    let res = tangent * map.x + bitangent * map.y + normal * map.z;

    return vec4<f32>(normalize(res), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    out.diffuse = textureSample(t_diffuse, s_diffuse, in.uv);
    // out.diffuse = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    out.normal = normal_mapping(in.normal, in.tangent, in.uv);
    // out.normal = vec4<f32>(in.normal, 1.0);
    out.pos = vec4<f32>(in.pos, 1.0);
    out.mr = textureSample(t_mr, s_mr, in.uv);

    return out;
}