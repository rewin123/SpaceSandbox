#version 450

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform sampler2D diffuse_tex;
layout(set = 0, binding = 1) uniform sampler2D normal_tex;

void main() {
    vec4 diff_color = texture(diffuse_tex, f_uv);
    vec3 normal = texture(normal_tex, f_uv).rgb;

    vec3 light_dir = normalize(vec3(1,1,1));

    float light_factor = dot(light_dir, normal);
    light_factor = max(0.2, light_factor);

    color = vec4(diff_color.rgb * light_factor, 1.0);
}