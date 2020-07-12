#version 330 core

layout (location = 0) in vec4 pos_tex;
layout (location = 1) in vec4 in_color;

uniform mat4 projection;

out vec4 frag_color;
out vec2 tex_coord;

void main() {
	gl_Position = projection * vec4(pos_tex.xy, 0.0, 1.0);
	frag_color = in_color;
	tex_coord = pos_tex.zw;
}
