#version 330 core

out vec4 out_color;

in vec4 frag_color;

void main() {
	out_color = frag_color;
}
