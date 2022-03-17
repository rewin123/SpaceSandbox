#version 450

layout(location = 0) out vec4 f_color;
layout(location = 1) out vec4 f_normal;
layout(location = 2) out vec4 f_cam_pos;

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 v_cam_pos;

layout(set = 1, binding  = 0) uniform sampler2D base_color; 

void main() {
    f_color = texture(base_color, v_uv);
    f_normal = vec4(v_normal, 1.0);
    f_cam_pos = vec4(v_cam_pos, 1.0);
}