#version 450

layout (location=0) out vec4 out_color;

layout (location=0) in vec4 screen_pos;
layout (location=1) in vec3 light_pos;
layout (location=2) in float intensity;
layout (location=3) in vec3 camera_pos;

layout(set=2,binding=0) uniform sampler2D color;
layout(set=3,binding=0) uniform sampler2D normal_tex;
layout(set=4,binding=0) uniform sampler2D metal_rough;
layout(set=5,binding=0) uniform sampler2D global_pos;

#define PI 3.14

float DistributionGGX(vec3 N, vec3 H, float a)
{
    float a2     = a*a;
    float NdotH  = max(dot(N, H), 0.0);
    float NdotH2 = NdotH*NdotH;

    float nom    = a2;
    float denom  = (NdotH2 * (a2 - 1.0) + 1.0);
    denom        = PI * denom * denom;

    return nom / denom;
}

float GeometrySchlickGGX(float NdotV, float k)
{
    float nom   = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float k)
{
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx1 = GeometrySchlickGGX(NdotV, k);
    float ggx2 = GeometrySchlickGGX(NdotL, k);

    return ggx1 * ggx2;
}

vec3 fresnelSchlick(float cosTheta, vec3 F0)
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void main() {
    float gamma = 2.2;
    //define ev
    vec2 uv_screen = vec2(screen_pos.x, screen_pos.y) / screen_pos.w;
    vec2 uv = (uv_screen + 1.0) / 2.0;

    //get texture data
    vec3 tex_color = texture(color, uv).rgb;
    vec3 pos = texture(global_pos, uv).rgb;
    vec3 N = texture(normal_tex, uv).rgb;
    vec3 mr = texture(metal_rough, uv).rgb;

    tex_color = pow(tex_color, vec3(gamma));
//    tex_color = pow(tex_color, vec3(gamme));
    mr = pow(mr, vec3(gamma));

    

//    mr.g = pow(mr.g, gamma);

    //get radiance
    float distance = length(light_pos - pos);
    float attenuation = 1.0 / (distance * distance);
    vec3 radiance = intensity * attenuation * vec3(1.0,1.0,1.0);

    vec3 L = normalize(light_pos - pos);
    vec3 V = normalize(camera_pos - pos);
    vec3 H = normalize(L + V);

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, tex_color, mr.r);

    float NDF = DistributionGGX(N, H, mr.g);
    float G   = GeometrySmith(N, V, L, mr.g);
    vec3 F    = fresnelSchlick(max(dot(H, V), 0.0), F0);

    vec3 kS = F;
    vec3 kD = vec3(1.0) - kS;
    kD *= 1.0 - mr.r;

    vec3 numerator = NDF * G * F;
    float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
    vec3 specular = numerator / max(denominator, 0.001);

    float NdotL = max(dot(N, L), 0.0);

    vec3 Lo = (kD * tex_color / PI + specular) * radiance * NdotL;
    Lo = pow(Lo, vec3(1.0 / gamma));

    out_color = vec4(Lo, 1.0);
}