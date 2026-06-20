#version 450

layout(push_constant) uniform PC {
    vec2 screen_size;
    vec2 cell_size;
} pc;

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_grid;
layout(location = 2) in vec4 in_atlas;
layout(location = 3) in vec4 in_fg;
layout(location = 4) in vec4 in_bg;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_fg;
layout(location = 2) out vec4 out_bg;

void main() {
    vec2 pixel = in_grid * pc.cell_size + in_pos * pc.cell_size;
    vec2 clip = vec2(
        2.0 * pixel.x / pc.screen_size.x - 1.0,
        1.0 - 2.0 * pixel.y / pc.screen_size.y
    );
    gl_Position = vec4(clip, 0.0, 1.0);
    out_uv = in_atlas.xy + in_pos * in_atlas.zw;
    out_fg = in_fg;
    out_bg = in_bg;
}
