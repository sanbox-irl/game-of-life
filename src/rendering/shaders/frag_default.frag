#version 450
// layout (location = 2) in vec3 frag_color;
layout (location = 1) in vec2 uv;
layout (location = 0) out vec4 color;

void main() {
    color = vec4(0.5, 0.2, 0.3, 1.0);
}