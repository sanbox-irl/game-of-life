#version 450
layout(push_constant) uniform FragPushConsts {
    layout(offset = 32) vec3 color;
    int selection_color;
}
push;

layout(location = 1) in vec2 uv;
layout(location = 2) in vec3 frag_color;

layout(location = 0) out vec4 color;

void main() {
    const int RIGHT = 1;
    const int UP = 2;
    const int LEFT = 4;
    const int DOWN = 8;
    const float MIN = 0.025;
    const float MAX = 1 - 0.025;
    const vec4 YELLOW = vec4(1.0, 0.88, 0.40, 1.0);

    if (uv.x < MIN) {
        if ((push.selection_color & LEFT) == LEFT) {
            color = YELLOW;
        } else {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    } else if (uv.x > MAX) {
        if ((push.selection_color & RIGHT) == RIGHT) {
            color = YELLOW;
        } else {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    } else if (uv.y < MIN) {
        if ((push.selection_color & DOWN) == DOWN) {
            color = YELLOW;
        } else {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    } else if (uv.y > MAX) {
        if ((push.selection_color & UP) == UP) {
            color = YELLOW;
        } else {
            color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    } else {
        color = vec4(push.color, 1.0);
    }
}