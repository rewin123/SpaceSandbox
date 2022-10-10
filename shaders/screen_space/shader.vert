#version 450

layout (location=0) in vec3 pos;

layout (location=0) out vec2 out_uv;

void main() {

    out_uv = vec2(pos.x + 1.0, pos.y + 1.0) / 2.0;
    gl_Position = vec4(pos, 1.0);
}