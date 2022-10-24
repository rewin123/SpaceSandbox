struct LightCamera {
    projection : mat4x4<f32>,
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
    @location(3) uv : vec2<f32>,
    @location(4) model_mat_1 : vec4<f32>,
    @location(5) model_mat_2 : vec4<f32>,
    @location(6) model_mat_3 : vec4<f32>,
    @location(7) model_mat_4 : vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos : vec3<f32>
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    var out : VertexOutput;
    let model_mat = mat4x4<f32>(model.model_mat_1,model.model_mat_2,model.model_mat_3,model.model_mat_4);

    let model_pos = model_mat * vec4<f32>(model.position, 1.0);
    var loc_pos = model_pos.rgb - camera.pos;
    var right = cross(camera.frw, camera.up);
    var view = vec3<f32>(dot(loc_pos, right), dot(loc_pos, camera.up), dot(loc_pos, camera.frw));
    var res = camera.projection * vec4<f32>(view, 1.0);
    out.clip_position = res;
    out.pos = loc_pos;
    return out;
}

struct FragmentOutput {
@builtin(frag_depth) depth : f32
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;

    out.depth = length(in.pos) / camera.far;

    return out;
}