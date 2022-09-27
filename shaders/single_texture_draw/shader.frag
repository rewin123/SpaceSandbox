#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec3 normal;
layout (location=1) in vec2 uv;

layout(set=1,binding=0) uniform sampler2D texturesampler;

void main() {
    vec3 light_dir = vec3(1, 1, 0);
    float light_val = dot(light_dir, normal);
    light_val = light_val > 0 ? light_val : 0;
    vec3 light_color = vec3(light_val * 0.7 + 0.3);
//    theColour = vec4(light_color, 1.0);
//    theColour = vec4(uv.x, uv.y, 0.5, 1.0);
    vec3 tex_color = texture(texturesampler, uv).rgb;

    theColour = vec4(tex_color * light_color, 1.0);
}