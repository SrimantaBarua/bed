#version 330 core

out vec4 out_color;

uniform sampler2D text;

in vec4 frag_color;
in vec2 tex_coord;

void main() {
	out_color = vec4(frag_color.xyz, frag_color.w * texture(text, tex_coord).r);
}
