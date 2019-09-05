#version 450
layout(push_constant) uniform FragPushConsts {
    layout(offset = 32) vec3 color;
    bool gridline_draw;
    float gridline_size;
    vec3 gridline_color;
}
push;

layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 frag_color;

layout(location = 0) out vec4 color;

void main() {
    float MIN = push.gridline_size;
    float MAX = 1.0 - push.gridline_size;

    if (push.gridline_draw &&
        (uv.x < MIN || uv.x > MAX || uv.y < MIN || uv.y > MAX)) {
        color = vec4(push.gridline_color.r, push.gridline_color.g, push.gridline_color.b, 1.0);
    } else {
        color = vec4(push.color, 1.0);
    }
}