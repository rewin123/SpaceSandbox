#version 450

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 f_uv;

layout(set = 0, binding = 0) uniform sampler2D tex;

layout(set = 1, binding = 0) uniform EyeData {
    float max_intensity;
} eye;

void main() {
    vec4 clr =  texture(tex, f_uv);


    color = vec4(clr.rgb / eye.max_intensity, clr.a);
}