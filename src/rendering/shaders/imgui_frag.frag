#version 450

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler font_sampler;

layout(location = 0) in vec2 uv;
layout(location = 1) in vec4 in_color;

layout(location = 0) out vec4 out_color;

void main() {
    out_color = in_color * texture(sampler2D(tex, font_sampler), uv);
}
