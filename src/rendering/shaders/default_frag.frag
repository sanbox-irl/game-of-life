#version 450
layout(push_constant) uniform FragPushConsts {
    layout(offset = 32) vec3 color;
    bool draw_gridlines;
}
push;

layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 frag_color;

layout(location = 0) out vec4 color;

void main() {
    const float MIN = 0.025;
    const float MAX = 1 - 0.025;

    if (push.draw_gridlines &&
        (uv.x < MIN || uv.x > MAX || uv.y < MIN || uv.y > MAX)) {
        color = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        color = vec4(push.color, 1.0);
    }
}