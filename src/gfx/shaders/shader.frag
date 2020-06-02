#version 450

layout(location = 0) in vec2 vpos;

layout(location = 0) out vec4 out_color;

layout(binding = 0) uniform usampler2D tex;

void main() {
	out_color = texture(tex, vpos);
	// out_color = vec4(1, 0, 0, 1);
}
