precision mediump float;

const float minRoughness = 0.04;
const float pi = 3.1415926535897932384626433832795;

struct MaterialInfo {
    vec4 baseColor;
    float metalFactor;
    float roughness;
};

struct SunLight {
    vec3 direction;
    vec3 color;
};

float fresnel(float metal, float viewDotHalf) {
    return metal + (1.0 - metal) * pow(clamp(1.0 - viewDotHalf, 0.0, 1.0), 5.0);
}

float schlickOcclusion(float perceptualRoughness, float lightDotHalf, float normalDotHalf)
{
    float k = perceptualRoughness * sqrt(2.0 / pi);

    float l = lightDotHalf / (lightDotHalf * (1.0 - k) + k);
    float n = normalDotHalf / (normalDotHalf * (1.0 - k) + k);
    return l * n;
}

float trowbridgeReitzMicrofacetDistribution(float alphaRoughness, float normalDotHalf)
{
    float roughnessSq = alphaRoughness * alphaRoughness;
    float f = (normalDotHalf * roughnessSq - normalDotHalf) * normalDotHalf + 1.0;
    return roughnessSq / (pi * f * f);
}

uniform mat4 view;

uniform MaterialInfo material;
uniform SunLight sun;
uniform vec3 ambient;

varying vec3 viewPos;
varying vec3 viewNormal;

void main() {
    vec3 normal = normalize(viewNormal);
    vec3 sunVect = -(view * vec4(sun.direction, 0.0)).xyz;
    vec3 viewVect = vec3(0.0, 0.0, 1.0); // Assumes an orthographic projection
    vec3 halfVect = normalize(sunVect + viewVect);

    float normalDotSun = clamp(dot(normal, sunVect), 0.001, 1.0);
    float normalDotView = clamp(abs(dot(normal, viewVect)), 0.001, 1.0);
    float normalDotHalf = clamp(dot(normal, halfVect), 0.0, 1.0);
    float sunDotHalf = clamp(dot(sunVect, halfVect), 0.0, 1.0);
    float viewDotHalf = clamp(dot(viewVect, halfVect), 0.0, 1.0);

    float perceptualRoughness = clamp(material.roughness, minRoughness, 1.0);

    float F = fresnel(material.metalFactor, viewDotHalf); // TODO: make Fresnel reflection white.
    float G = schlickOcclusion(perceptualRoughness, sunDotHalf, normalDotHalf);
    float D = trowbridgeReitzMicrofacetDistribution(perceptualRoughness * perceptualRoughness, normalDotHalf);
    float specular = (F * G * D) / (4.0 * normalDotSun * normalDotView);
    float diffuse = 1.0 / pi; // Standard Lambert diffuse term
    vec3 reflectance = material.baseColor.xyz * (diffuse + specular);

    vec3 reflectedSun = max(normalDotSun, 0.0) * sun.color * reflectance;
    vec3 reflectedAmbient = ambient * material.baseColor.xyz;

    gl_FragColor = vec4(reflectedSun + reflectedAmbient, material.baseColor.w);
}
