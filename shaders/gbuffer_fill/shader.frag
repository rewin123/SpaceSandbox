#version 450

layout (location=0) out vec4 out_color;
layout (location=1) out vec4 out_normal;
layout (location=2) out vec4 out_metal_rough;
layout (location=3) out vec4 out_pos;

layout (location=0) in vec3 normal;
layout (location=1) in vec2 uv;
layout (location=2) in vec4 in_pos;
layout (location=3) in vec3 tangent;

layout(set=1,binding=0) uniform sampler2D color;
layout(set=2,binding=0) uniform sampler2D normal_tex;
layout(set=3,binding=0) uniform sampler2D metal_rough;

void main() {
    vec4 tex_color = texture(color, uv);
    vec4 tex_normal = texture(normal_tex, uv);
    vec3 tex_metal_rough = texture(metal_rough, uv).rgb;

    vec3 ny = -cross(normal, tangent);

    tex_normal = tex_normal * 2.0 - 1.0;

    vec3 real_normal = normalize(normal * tex_normal.z + tangent * tex_normal.x + ny * tex_normal.y);
    real_normal = real_normal * tex_normal.w + (1.0 - tex_normal.w) * normal;
    if (tex_color.a < 0.5) {
        discard;
    }

    out_color = vec4(tex_color.rgb, 1.0);
    out_normal = vec4(real_normal, 1.0);
    out_metal_rough = vec4(tex_metal_rough, 1.0);
    out_pos = vec4(in_pos.rgb, 1.0);
}