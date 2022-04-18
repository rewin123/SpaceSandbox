#version 450

layout(location = 0) out vec4 f_depth;

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 v_cam_pos;

void main() {
    f_depth = vec4(v_cam_pos, 1.0);
}