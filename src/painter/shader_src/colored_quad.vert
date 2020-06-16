#version 330 core

layout (location = 0) in vec2 pos;
layout (location = 1) in vec4 in_color;

uniform mat4 projection;

out vec4 frag_color;

void main() {
	gl_Position = projection * vec4(pos, 0.0, 1.0);
	frag_color = in_color;
}
