precision mediump float;

const float minRoughness = 0.04;
const float pi = 3.1415926535897932384626433832795;

struct MaterialInfo {
    vec4 baseColor;
    float metalFactor;
    float roughness;
};

struct PointLight {
    vec3 position;
    vec3 color;
};

float fresnel(float metal, float viewDotHalf) {
    return metal + (1.0 - metal) * pow(clamp(1.0 - viewDotHalf, 0.0, 1.0), 5.0);
}

float schlickOcclusion(float perceptualRoughness, float lightDotHalf, float normalDotHalf)
{
    float k = perceptualRoughness * 0.79788;

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

uniform MaterialInfo material;
uniform PointLight light;

varying vec3 viewPos;
varying vec3 viewNormal;

void main() {
    vec3 normal = normalize(viewNormal);

    vec3 lightPos = light.position;
    vec3 lightVect = viewPos - lightPos;
    float distToLight = length(lightVect);
    vec3 lightVectNormalized = lightVect / distToLight;
    vec3 viewVect = normalize(-viewPos);
    vec3 halfVect = normalize(lightVectNormalized + viewVect);

    float normalDotLight = dot(normal, lightVectNormalized);
    float lightDotHalf = dot(lightVectNormalized, halfVect);
    float normalDotHalf = dot(normal, halfVect);
    float normalDotView = dot(normal, viewVect);
    float viewDotHalf = dot(viewVect, halfVect);

    float perceptualRoughness = clamp(material.roughness, minRoughness, 1.0);

    float specular = fresnel(material.metalFactor, viewDotHalf)
        * schlickOcclusion(perceptualRoughness, lightDotHalf, normalDotHalf)
        * trowbridgeReitzMicrofacetDistribution(perceptualRoughness * perceptualRoughness, normalDotHalf)
        / (4.0 * normalDotLight * normalDotView);
    float diffuse = 1.0 / pi; // Standard Lambert diffuse term
    vec3 reflectance = material.baseColor.xyz * (diffuse + specular);

    vec3 reflectedLight = max(normalDotLight, 0.0) * light.color * reflectance;

    gl_FragColor = vec4(reflectedLight, material.baseColor.w);
}
