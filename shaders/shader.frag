#version 450

layout (location=0) out vec4 theColour;

layout (location=0) in vec3 normal;

void main(){
    vec3 light_dir = vec3(0, 1, 0);
    float light_val = dot(light_dir, normal);
    light_val = light_val > 0 ? light_val : 0;
    theColour= vec4(vec3(light_val * 0.9 + 0.1),1.0);
}