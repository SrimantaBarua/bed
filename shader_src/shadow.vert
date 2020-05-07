#version 330 core

layout (location = 0) in vec4 pos_tex;

uniform mat4 projection;

out vec2 tex_coord;

void main() {
	gl_Position = projection * vec4(pos_tex.xy, 0.0, 1.0);
	tex_coord = pos_tex.zw;
}

