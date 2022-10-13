#version 450

layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec3 tangent;
layout (location=3) in vec2 uv;
layout (location=4) in mat4 model_matrix;

layout (set=0, binding=0) uniform UniformBufferObject {
    mat4 view_matrix;
    mat4 projection_matrix;
} ubo;

#define PI 3.14

layout (location=0) out vec4 model_pos;

void main() {

    model_pos = model_matrix * vec4(position, 1.0);
    vec4 res = ubo.projection_matrix * ubo.view_matrix * model_matrix * vec4(position, 1.0);
    res.x *= -1;
    gl_Position = res;
}