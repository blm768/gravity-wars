precision mediump float;

//const float minRoughness = 0.04;
const float pi = 3.1415926535897932384626433832795;

struct PointLight {
    vec3 position;
    vec3 color;
};

float fresnel(float metal, float lightVectDotNormal) {
    return metal + (1.0 - metal) * pow(1.0 - lightVectDotNormal, 5.0);
}

uniform vec4 baseColor;
uniform PointLight light;

varying vec3 viewPos;
varying vec3 viewNormal;

void main() {
    vec3 normal = normalize(viewNormal);

    vec3 lightPos = light.position;
    vec3 lightVect = viewPos - lightPos;
    float distToLight = length(lightVect);
    vec3 lightVectNormalized = lightVect / distToLight;

    float normalDotLight = dot(normal, lightVectNormalized);

    float specular = fresnel(0.5, normalDotLight);
    float diffuse = 1.0 / pi;
    gl_FragColor = vec4(max(normalDotLight, 0.0) * baseColor.xyz * (diffuse + specular), baseColor.w);
}
