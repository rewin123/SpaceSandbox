#version 450

layout (location=0) in vec4 position;
layout (location=1) in vec3 normal;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout (location=0) out vec3 out_normal;

void main() {
    out_normal = normal;
    gl_Position = ubo.projection_matrix * ubo.view_matrix * position;
}