#version 450
layout (push_constant) uniform PushConsts {
    vec3 color;
} push;

layout (location = 1) in vec2 uv;
layout (location = 2) in vec3 frag_color;

layout (location = 0) out vec4 color;

void main() {
    color = vec4(push.color, 1.0);
}