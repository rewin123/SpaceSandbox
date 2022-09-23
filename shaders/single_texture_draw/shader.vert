#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uv;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout (location=0) out vec3 out_normal;
layout (location=1) out vec2 out_uv;

void main() {
    out_normal = normal;
    out_uv = uv;
    gl_Position = ubo.projection_matrix * ubo.view_matrix * vec4(position, 1.0);
}