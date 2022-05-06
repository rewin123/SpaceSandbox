#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;
layout(location = 2) out vec3 v_cam_pos;

layout(set = 0, binding = 0) uniform Data {
    vec3 forward;
    vec3 up;
    vec3 cam_pos;
} uniforms;

void main() {
    vec3 local_pos = position - uniforms.cam_pos;
    float f = dot(uniforms.forward, local_pos);
    float u = dot(uniforms.up, local_pos);
    float r = dot(cross(uniforms.up, uniforms.forward), local_pos);
    vec3 cam_pos = vec3(r, u, f);
    v_normal = normal;
    v_uv = uv;
    v_cam_pos = cam_pos;
    gl_Position = vec4((cam_pos) / 50.0 + vec3(0.0, 0.0, 0.5), 1.0);
}