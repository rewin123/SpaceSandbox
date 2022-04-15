#version 450

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform sampler2D tex;
layout(set = 0, binding = 1) uniform sampler2D max_tex;

void main() {
    vec4 clr =  texture(tex, f_uv);
    vec3 local_emissive = texture(max_tex, f_uv).rgb;
    float mav = max(local_emissive.r, max(local_emissive.g, local_emissive.b));
    color = vec4(clr.rgb / mav, clr.a);
}