#version 450

layout(location = 0) in vec2 in_pos;

layout(location = 0) out vec2 out_pos;

layout(binding = 0) uniform usampler2D tex;

// TODO: use ints
layout(push_constant) uniform PushConsts {
	vec2 pos;
	vec2 win_size;
} pc;

void main() {
	out_pos = in_pos;
	gl_Position = vec4((in_pos * textureSize(tex, 0) + pc.pos) / pc.win_size * 2 - 1, 0.0, 1.0);
}
