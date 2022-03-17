#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec3 v_cam_pos;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec3 cam_pos;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = normal;
    v_uv = uv;
    v_cam_pos = position - uniforms.cam_pos;
    gl_Position = uniforms.proj * worldview * vec4(position, 1.0);
}