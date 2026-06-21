#version 450

layout(location = 0) in vec2 in_uv;
layout(location = 1) in vec4 in_fg;
layout(location = 2) in vec4 in_bg;
layout(location = 3) in vec3 in_light;

layout(binding = 0) uniform sampler2D atlas;

layout(location = 0) out vec4 out_color;

void main() {
    float alpha = texture(atlas, in_uv).r;
    vec4 color = mix(in_bg, in_fg, alpha);
    out_color = vec4(color.rgb * in_light, color.a);
}
