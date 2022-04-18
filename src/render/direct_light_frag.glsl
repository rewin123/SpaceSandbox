#version 450

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform sampler2D diffuse_tex;
layout(set = 0, binding = 1) uniform sampler2D normal_tex;
layout(set = 0, binding = 2) uniform sampler2D pos_tex;
layout(set = 0, binding = 3) uniform sampler2D shadow_tex;

layout(set = 1, binding = 0) uniform LightData {
    vec3 direction;
    vec3 color;
    vec3 cam_pos;
    vec3 light_pos;
} light;

layout(set = 1, binding = 1) uniform LightPosData {
    mat4 world;
    mat4 view;
    mat4 proj;
    vec3 cam_pos;
} uniforms;

void main() {
    vec4 diff_color = texture(diffuse_tex, f_uv);
    vec3 normal = texture(normal_tex, f_uv).rgb;

    vec3 pos = texture(pos_tex, f_uv).rgb + light.cam_pos;

    vec4 shadow_sampled = uniforms.proj * uniforms.view * uniforms.world * vec4(pos, 1.0);
    shadow_sampled.x = (shadow_sampled.x + 1.0) * 0.5;
    shadow_sampled.y = (shadow_sampled.y + 1.0) * 0.5;

    vec3 shadow_pos = texture(shadow_tex, vec2(shadow_sampled.x, shadow_sampled.y)).rgb;
    vec3 pos_in_dir = pos - light.light_pos;

    float shadow_dist = -dot(shadow_pos, light.direction);
    float pos_dist = -dot(pos_in_dir, light.direction);

    if (pos_dist <= shadow_dist) {
        float light_factor = dot(light.direction.rgb, normal);
        light_factor = max(0.2, light_factor);
        color = vec4(diff_color.rgb * light_factor * light.color.rgb, 1.0);
    } else {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    }

    
}