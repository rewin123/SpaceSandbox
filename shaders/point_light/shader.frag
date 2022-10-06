#version 450

layout (location=0) out vec4 out_color;

layout (location=0) in vec4 screen_pos;
layout (location=1) in vec3 light_pos;

layout(set=2,binding=0) uniform sampler2D color;
layout(set=3,binding=0) uniform sampler2D normal_tex;
layout(set=4,binding=0) uniform sampler2D metal_rough;
layout(set=5,binding=0) uniform sampler2D global_pos;

float checker(float val) {
    return val;
}

void main() {
    vec2 uv_screen = vec2(screen_pos.x, screen_pos.y) / screen_pos.w;
    vec2 uv = (uv_screen + 1.0) / 2.0;
    vec3 tex_color = texture(color, uv).rgb;
    vec3 pos = texture(global_pos, uv).rgb;
    vec3 normal = texture(normal_tex, uv).rgb;

    vec3 light_dir = normalize(light_pos - pos);
    float k = dot(light_dir, normal);
    k = max(0.0, k);
    k = 0.2 + 0.8 * k;

    out_color = vec4(tex_color * k, 1.0);
}