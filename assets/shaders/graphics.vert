#version 450

layout(push_constant) uniform PC {
    vec2 screen_size;
    vec2 cell_size;
} pc;

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_grid;
layout(location = 2) in vec4 in_color;

layout(location = 0) out vec4 out_color;

void main() {
    vec2 pixel = (in_grid + in_pos) * pc.cell_size;
    gl_Position = vec4(
        2.0 * pixel.x / pc.screen_size.x - 1.0,
        2.0 * pixel.y / pc.screen_size.y - 1.0,
        0.0, 1.0
    );
    out_color = in_color;
}
