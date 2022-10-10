#version 450

layout (location=0) out vec4 out_color;

layout (location=0) in vec2 uv;

layout(set=0,binding=0) uniform sampler2D tex_color;

void main() {
    float gamma = 2.2;

    vec3 Lo = texture(tex_color, uv).rgb;

    Lo = pow(Lo, vec3(1.0 / gamma));

    out_color = vec4(Lo, 1.0);
}