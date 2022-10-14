struct CameraUniform {
    view : mat4x4<f32>,
    proj : mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> camera : CameraUniform;

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) tangent : vec3<f32>,
    @location(3) uv : vec3<f32>,
    @location(4) model_matrix : mat4x4<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec2<f32>,
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    var out : VertexOutput;
    out.normal = model.normal;
    out.clip_position = camera.proj * camera.view * model_matrix * vec4<f32>(model.position, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.normal;
}