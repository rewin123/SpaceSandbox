struct LightCamera {
    pos : vec3<f32>,
    frw : vec3<f32>,
    up : vec3<f32>,
    far : f32,
};

@group(0) @binding(0)
var<uniform> camera : LightCamera;

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) normal : vec3<f32>,
    @location(2) tangent : vec3<f32>,
    @location(3) uv : vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    var out : VertexOutput;

    var loc_pos = model.position - camera.pos;
    var right = cross(camera.frw, camera.up);
    var view = vec3<f32>(dot(loc_pos, right), dot(loc_pos, camera.up), dot(loc_pos, camera.frw));

    if (abs(view.z) > 0.01) {
        view.x /= view.z;
        view.y /= view.z;
        view.z /= camera.far;
    }
    out.clip_position = vec4<f32>(view, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) {

}