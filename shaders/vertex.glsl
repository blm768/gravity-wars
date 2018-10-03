precision mediump float;

attribute vec3 position;
attribute vec3 normal;

uniform mat4 modelView; // TODO: split.
uniform mat4 projection;

varying vec3 viewPos;
varying vec3 viewNormal;

void main() {
    viewPos = (modelView * vec4(position, 1.0)).xyz;
    viewNormal = (modelView * vec4(normal, 1.0)).xyz;
    gl_Position = projection * vec4(viewPos, 1.0);
}
