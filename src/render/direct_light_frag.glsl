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
    vec3 forward;
    vec3 up;
    vec3 cam_pos;
} light_cam;

vec3 GetDirLightPos(vec3 global_pos) {
    vec3 local_pos = global_pos - light_cam.cam_pos;
    float f = dot(light_cam.forward, local_pos);
    float u = dot(light_cam.up, local_pos);
    float r = dot(cross(light_cam.up, light_cam.forward), local_pos);
    vec3 cam_pos = vec3(r, u, f);
    return cam_pos;
}

void main() {
    vec4 diff_color = texture(diffuse_tex, f_uv);
    vec3 normal = texture(normal_tex, f_uv).rgb;

    vec3 pos = texture(pos_tex, f_uv).rgb;
    vec4 shadow_val = texture(shadow_tex, f_uv);

    vec3 pixel_light_pos = GetDirLightPos(pos + light.cam_pos);
    vec2 uv = pixel_light_pos.rg / 50.0;
    uv.x = uv.x * 0.5 + 0.5;
    uv.y = uv.y * 0.5 + 0.5;
    vec3 shadow_pos = texture(shadow_tex, uv).rgb;

    vec3 dd = light.direction;
    vec3 ll = light_cam.forward;

    float light_factor = dot(-light.direction, normal);

    vec3 ambient = diff_color.rgb * light.color * 0.5;

    if (abs(shadow_pos.z - pixel_light_pos.z) > 0.1 || light_factor <= 0) {

        color = vec4(ambient, 1.0);
    }
    else {
        color = vec4(ambient + light_factor * diff_color.rgb * light.color, 1.0);
    }
    
    // color = vec4(shadow_pos, 1.0);
}