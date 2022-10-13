#version 450

layout (location=0) in vec4 model_pos;

layout (set = 1, binding = 0) uniform LightShadowUniform {
    vec3 light_pos;
} light;

void main() {

    float lightDistance = length(model_pos.xyz - light.light_pos);

    // map to [0;1] range by dividing by far_plane
    lightDistance = lightDistance / 100.0;

    // write this as modified depth
    gl_FragDepth = lightDistance;

}