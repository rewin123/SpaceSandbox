#version 450

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform sampler2D tex;
layout(set = 1, binding = 0) uniform ScaleData {
    vec2 step;
} data;

void main() {
    vec4 mav = texture(tex, f_uv);
    mav = max(mav, texture(tex, f_uv + vec2(data.step.x, 0.0)));
    mav = max(mav, texture(tex, f_uv + vec2(0.0, data.step.y)));
    mav = max(mav, texture(tex, f_uv + data.step));
    color = mav;
}