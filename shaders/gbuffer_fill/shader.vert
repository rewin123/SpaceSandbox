#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uv;
layout (location=3) in mat4 model_matrix;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

layout (location=0) out vec3 out_normal;
layout (location=1) out vec2 out_uv;
layout (location=2) out vec4 out_pos;

void main() {
    out_normal = normal;
    out_uv = uv;
    out_pos = model_matrix * vec4(position, 1.0);
    gl_Position = ubo.projection_matrix * ubo.view_matrix * model_matrix * vec4(position, 1.0);
}