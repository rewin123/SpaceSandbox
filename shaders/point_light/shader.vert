#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uv;
layout (location=3) in float intensity;
layout (location=4) in vec3 color;
layout (location=5) in vec3 pos;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
    vec3 camera_pos;
} ubo;

layout (set=1, binding=0) uniform LightInfo {
    vec2 screen_size;
} info;

layout (location=0) out vec4 out_pos;
layout (location=1) out vec3 light_pos;
layout (location=2) out float out_intensity;
layout (location=3) out vec3 out_camera_pos;

void main() {
    float scale = intensity * 100;
    vec4 screen_pos = ubo.projection_matrix * ubo.view_matrix * vec4(position * scale + pos, 1.0);
    gl_Position = screen_pos;
    out_pos = screen_pos;
    light_pos = pos;
    out_intensity = intensity;
    out_camera_pos = ubo.camera_pos;
}