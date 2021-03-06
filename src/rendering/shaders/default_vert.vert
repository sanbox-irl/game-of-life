#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 vert_uv;
layout(location = 2) in vec3 color;

layout(location = 0) out gl_PerVertex { vec4 gl_Position; };
layout(location = 1) out vec2 frag_uv;
layout(location = 2) out vec3 frag_color;

layout(push_constant) uniform PushConsts {
  vec2 entity_position;
  vec2 camera_position;
  float scale;
  float aspect_ratio;
}
push;

void main() {
  vec2 model_position =
      vec2(push.entity_position.x + position.x,
           (push.entity_position.y + position.y) * push.aspect_ratio);
  vec4 object_space_pos =
      vec4(model_position.x / push.scale - push.camera_position.x,
           model_position.y / push.scale - push.camera_position.y, 1.0, 1.0);
  gl_Position = object_space_pos;

  frag_uv = vert_uv;
  frag_color = color;
}