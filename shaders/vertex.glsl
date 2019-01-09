precision mediump float;

attribute vec3 position;
attribute vec3 normal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

varying vec3 viewPos;
varying vec3 viewNormal;

void main() {
    mat4 modelView = view * model;
    viewPos = (modelView * vec4(position, 1.0)).xyz;
    viewNormal = (modelView * vec4(normal, 0.0)).xyz; // Assumes that modelView is orthogonal
    gl_Position = projection * vec4(viewPos, 1.0);
}
