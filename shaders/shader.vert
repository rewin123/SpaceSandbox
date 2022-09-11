#version 450

layout (location=0) in vec4 position;

void main() {
    gl_PointSize=20.0;
    gl_Position = position;
}