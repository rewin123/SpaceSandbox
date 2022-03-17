#version 450

layout(location = 0) out vec2 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform texture2D tex;

void main() {
    color = texture(tex, f_uv);
}