layout (location=0) in vec3 position;
layout (location=1) in vec3 normal;
layout (location=2) in vec2 uv;
layout (location=3) in mat4 model_matrix;

#define PI 3.14

uniform LightData {
    vec3 pos;
    float intensity;
} light;

vec3 get_sphere(vec3 pos) {
    float z = length(dp);

    float r = sqrt(dp.x * dp.x + dp.y * dp.y);

    float phi = acos(dp.x / r);
    phi -= dp.y > 0 ? 0.0 : PI;

    float teta = asin(dp.z / z);

    return vec3(phi, teta, z);
}

vec3 get_sphere_viewport(vec3 pos, float dist) {
    vec3 sphere = get_sphere(pos);
    sphere.x /= PI;
    sphere.y /= PI;
    sphere.z /= dist;
    return sphere;
}

void main() {
    vec4 vertex_pos = model_matrix * vec4(position, 1.0);

    vec3 dp = vertex_pos.rgb - light.pos;

    gl_Position = vec4(get_sphere_viewport(dp, intensity), 1.0);
}