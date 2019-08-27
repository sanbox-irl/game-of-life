#version 450
layout(push_constant) uniform FragPushConsts { layout(offset = 32) vec3 color; }
push;

layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 frag_color;

layout(location = 0) out vec4 color;

void main() {
  if (uv.x < 0.025 || uv.x > 1 - 0.025 || uv.y < 0.025 || uv.y > 1 - 0.025) {
    color = vec4(0.0, 0.0, 0.0, 1.0);
  } else {
    color = vec4(push.color, 1.0);
  }
}