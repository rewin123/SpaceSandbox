#version 450

layout (location=0) in vec4 position;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
} ubo;


void main() {
    gl_Position = ubo.view_matrix * position;
}