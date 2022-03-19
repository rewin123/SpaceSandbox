#version 450

layout(location = 0) in vec2 uv;

layout (location = 0) out vec2 f_uv;

void main() {
    gl_Position = vec4(uv.x * 2.0 - 1.0, uv.y * 2.0 - 1.0, 0.0, 1.0);
    f_uv = uv;
}