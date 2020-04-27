#version 450

layout(location = 0) in vec2 in_pos;

layout(location = 0) out vec2 out_pos;

layout(push_constant) uniform PushConsts {
	vec2 pos;
} pc;

void main() {
	gl_Position = vec4(pc.pos + in_pos, 0.0, 1.0);
	out_pos = in_pos;
}
