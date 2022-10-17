struct CameraUniform {
    view : mat4x4<f32>,
    proj : mat4x4<f32>,
    pos : vec3<f32>
};

struct PointLightUniform {
    position : vec3<f32>,
    color : vec3<f32>,
    intensity : f32
}

@group(0) @binding(0)
var<uniform> camera : CameraUniform;

@group(1) @binding(0)
var<uniform> light : PointLightUniform;

@group(1) @binding(1)
var t_shadow: texture_cube<f32>;
@group(1) @binding(2)
var s_shadow: sampler;

struct VertexInput {
    @location(0) position : vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pos: vec4<f32>,
}

@vertex
fn vs_main(
    model : VertexInput
) -> VertexOutput {
    var out : VertexOutput;
    out.clip_position = camera.view * vec4<f32>(model.position * sqrt(light.intensity / pow(0.01, 2.2)), 1.0);
    out.clip_position.z = min(-0.1, out.clip_position.z);
    out.clip_position = camera.proj * out.clip_position;
    out.pos = out.clip_position;
    return out;
}

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@group(2) @binding(2)
var t_normal: texture_2d<f32>;
@group(2) @binding(3)
var s_normal: sampler;

@group(2) @binding(4)
var t_position: texture_2d<f32>;
@group(2) @binding(5)
var s_position: sampler;

struct FragmentOutput {
@location(0) color : vec4<f32>,
};


let PI = 3.14;

fn DistributionGGX(N : vec3<f32>, H : vec3<f32>, a : f32) -> f32
{
    var a2     = a*a;
    var NdotH  = max(dot(N, H), 0.0);
    var NdotH2 = NdotH*NdotH;

    var nom    = a2;
    var denom  = (NdotH2 * (a2 - 1.0) + 1.0);
    denom        = PI * denom * denom;

    return nom / denom;
}

fn GeometrySchlickGGX(NdotV : f32, k : f32) -> f32
{
    var nom   = NdotV;
    var denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

fn GeometrySmith(N : vec3<f32>, V : vec3<f32>, L : vec3<f32>,  k : f32) -> f32
{
    var NdotV = max(dot(N, V), 0.0);
    var NdotL = max(dot(N, L), 0.0);
    var ggx1 = GeometrySchlickGGX(NdotV, k);
    var ggx2 = GeometrySchlickGGX(NdotL, k);

    return ggx1 * ggx2;
}

fn fresnelSchlick( cosTheta : f32, F0 : vec3<f32>) -> vec3<f32>
{
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}


fn sample_shadow(dir : vec3<f32>, N : vec3<f32>, T : vec3<f32>, dist : f32) -> f32 {
    var res : f32 = 0.0;
    var up = cross(N, T);

    for (var dx : i32 = -2; dx < 3; dx++) {
        for (var dy : i32 = -2; dy < 3; dy++) {
            var k = dir + (up * f32(dy) + T * f32(dx)) / 512.0;
            var shadow_dist = textureSample(t_shadow, s_shadow, normalize(k)).r;
            if (dist / 10000.0 > shadow_dist + 0.00001) {
                
            } else {
                res += 1.0;
            }
        }
    }

    res /= 25.0;
    return res;
}

@fragment
fn fs_main(in: VertexOutput) -> FragmentOutput {
    var out : FragmentOutput;
    
    var screen_uv = vec2<f32>(in.pos.x / in.pos.w / 2.0 + 0.5, -in.pos.y / in.pos.w / 2.0 + 0.5);

    var N = textureSample(t_normal, s_diffuse, screen_uv).rgb;
    var pos = textureSample(t_position, s_position, screen_uv).rgb;
    var L = normalize(light.position - pos);
    var V = normalize(camera.pos - pos);
    var dist = length(light.position - pos);

    // var shadow_dist = textureSample(t_shadow, s_shadow, -L).r;
    var shadow = sample_shadow(-L, N, normalize(cross(L, N)), dist);
    // if (dist / 10000.0 > shadow_dist + 0.00001) {
    //     shadow = 0.0;
    // }

    var tex_color = textureSample(t_diffuse, s_diffuse, screen_uv).rgb;
    var mr = vec3(0.0, 0.5, 0.0);
    var attenuation = 1.0 / (dist * dist);
    var radiance = light.intensity * attenuation * vec3(1.0,1.0,1.0);
    var H = normalize(L + V);

    var F0 = vec3<f32>(0.04,0.04,0.04);
    F0 = mix(F0, tex_color, mr.r);

    var NDF = DistributionGGX(N, H, mr.g);
    var G   = GeometrySmith(N, V, L, mr.g);
    var F   = fresnelSchlick(max(dot(H, V), 0.0), F0);

    var kS = F;
    var kD = vec3(1.0) - kS;
    kD *= 1.0 - mr.r;

    var numerator = NDF * G * F;
    var denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
    var specular = numerator / max(denominator, 0.001);

    var NdotL = max(dot(N, L), 0.0);

    var Lo = (kD * tex_color / PI + specular) * radiance * NdotL;
    Lo = Lo * shadow;
    Lo = pow(Lo, vec3<f32>(1.0 / 2.2));
    out.color = vec4<f32>(Lo, 1.0);
//    out.color = vec4<f32>(shadow_dist,shadow_dist,shadow_dist,1.0);
//    out.color = vec4<f32>(abs(-L.x), abs(-L.x),abs(-L.x),1.0);
    return out;
}