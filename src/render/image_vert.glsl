#version 450

layout (location = 0) vec2 v_uv;

layout (location = 0) vec2 f_uv;

void main() {
    gl_Position = vec4(v_uv.x * 2.0 - 1.0, v_uv.y * 2.0 - 1.0, 0.0, 1.0);
    f_uv = v_uv;
}