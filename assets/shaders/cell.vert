#version 450

layout(push_constant) uniform PC {
    vec2 screen_size;
    vec2 cell_size;
    ivec2 world_size;
    ivec2 cam_pos;
    vec3 ambient;
    uint is_ui;
    uint light_count;
} pc;

layout(location = 0) in vec2 in_pos;
layout(location = 1) in vec2 in_grid;
layout(location = 2) in vec4 in_atlas;
layout(location = 3) in vec4 in_fg;
layout(location = 4) in vec4 in_bg;

layout(location = 0) out vec2 out_uv;
layout(location = 1) out vec4 out_fg;
layout(location = 2) out vec4 out_bg;
layout(location = 3) out vec3 out_light;

layout(std430, binding = 1) readonly buffer GridBuffer {
    uint cells[];
} grid;

struct LightSrc {
    vec2 pos;
    float radius;
    float pad0;
    vec3 color;
    float pad1;
};

layout(std430, binding = 2) readonly buffer LightBuffer {
    LightSrc sources[];
} lights;

bool is_solid(uint m) {
    return m == 3u || m == 5u || m == 6u || m == 7u || m == 12u || m == 13u || m == 14u;
}

bool line_of_sight(ivec2 a, ivec2 b) {
    ivec2 p = a;
    ivec2 d = abs(b - a);
    ivec2 s = ivec2(a.x < b.x ? 1 : -1, a.y < b.y ? 1 : -1);
    int err = d.x - d.y;
    while (true) {
        if (p == b) return true;
        ivec2 vp = p - pc.cam_pos;
        if (vp.x < 0 || vp.x >= pc.world_size.x || vp.y < 0 || vp.y >= pc.world_size.y) return false;
        uint m = grid.cells[vp.y * pc.world_size.x + vp.x];
        if (is_solid(m)) return false;
        int e2 = 2 * err;
        if (e2 > -d.y) { err -= d.y; p.x += s.x; }
        if (e2 < d.x) { err += d.x; p.y += s.y; }
    }
}

vec3 compute_light(ivec2 world_pos) {
    vec3 light = pc.ambient;
    for (uint i = 0u; i < pc.light_count; i++) {
        LightSrc src = lights.sources[i];
        ivec2 src_pos = ivec2(src.pos);
        float dist = length(vec2(world_pos - src_pos));
        float rad = src.radius;
        if (dist >= rad) continue;
        if (!line_of_sight(src_pos, world_pos)) continue;
        float t = 1.0 - dist / rad;
        float att = t * t;
        light += src.color * att;
    }
    return min(light, vec3(1.0));
}

void main() {
    vec2 pixel = (in_grid + in_pos) * pc.cell_size;
    gl_Position = vec4(
        2.0 * pixel.x / pc.screen_size.x - 1.0,
        2.0 * pixel.y / pc.screen_size.y - 1.0,
        0.0, 1.0
    );
    out_uv = in_atlas.xy + in_pos * in_atlas.zw;
    out_fg = in_fg;
    out_bg = in_bg;
    if (pc.is_ui != 0u) {
        out_light = vec3(1.0);
    } else {
        ivec2 world_pos = ivec2(in_grid + vec2(pc.cam_pos));
        out_light = compute_light(world_pos);
    }
}
