#version 450

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform sampler2D tex;

void main() {
    vec4 mav = texture(tex, f_uv);

    ivec2 tex_size = textureSize(tex, 0);
    vec2 step = vec2(1.f, 1.f) / vec2(tex_size);

    mav = max(mav, texture(tex, f_uv + vec2(step.x, 0.0)));
    mav = max(mav, texture(tex, f_uv + vec2(0.0, step.y)));
    mav = max(mav, texture(tex, f_uv + step));
    color = mav;
}