#version 330 core

out vec4 out_color;

uniform sampler2D tex;

in vec2 tex_coord;

void main() {
	vec2 offset = 1.0 / textureSize(tex, 0);

	vec2 offsets[49] = vec2[](
		vec2(-3.0 * offset.x, -3.0 * offset.y),
		vec2(-2.0 * offset.x, -3.0 * offset.y),
		vec2(-1.0 * offset.x, -3.0 * offset.y),
		vec2( 0.0           , -3.0 * offset.y),
		vec2( 1.0 * offset.x, -3.0 * offset.y),
		vec2( 2.0 * offset.x, -3.0 * offset.y),
		vec2( 3.0 * offset.x, -3.0 * offset.y),

		vec2(-3.0 * offset.x, -2.0 * offset.y),
		vec2(-2.0 * offset.x, -2.0 * offset.y),
		vec2(-1.0 * offset.x, -2.0 * offset.y),
		vec2( 0.0           , -2.0 * offset.y),
		vec2( 1.0 * offset.x, -2.0 * offset.y),
		vec2( 2.0 * offset.x, -2.0 * offset.y),
		vec2( 3.0 * offset.x, -2.0 * offset.y),

		vec2(-3.0 * offset.x, -1.0 * offset.y),
		vec2(-2.0 * offset.x, -1.0 * offset.y),
		vec2(-1.0 * offset.x, -1.0 * offset.y),
		vec2( 0.0           , -1.0 * offset.y),
		vec2( 1.0 * offset.x, -1.0 * offset.y),
		vec2( 2.0 * offset.x, -1.0 * offset.y),
		vec2( 3.0 * offset.x, -1.0 * offset.y),

		vec2(-3.0 * offset.x, 0.0),
		vec2(-2.0 * offset.x, 0.0),
		vec2(-1.0 * offset.x, 0.0),
		vec2( 0.0           , 0.0),
		vec2( 1.0 * offset.x, 0.0),
		vec2( 2.0 * offset.x, 0.0),
		vec2( 3.0 * offset.x, 0.0),

		vec2(-3.0 * offset.x, 1.0 * offset.y),
		vec2(-2.0 * offset.x, 1.0 * offset.y),
		vec2(-1.0 * offset.x, 1.0 * offset.y),
		vec2( 0.0           , 1.0 * offset.y),
		vec2( 1.0 * offset.x, 1.0 * offset.y),
		vec2( 2.0 * offset.x, 1.0 * offset.y),
		vec2( 3.0 * offset.x, 1.0 * offset.y),

		vec2(-3.0 * offset.x, 2.0 * offset.y),
		vec2(-2.0 * offset.x, 2.0 * offset.y),
		vec2(-1.0 * offset.x, 2.0 * offset.y),
		vec2( 0.0           , 2.0 * offset.y),
		vec2( 1.0 * offset.x, 2.0 * offset.y),
		vec2( 2.0 * offset.x, 2.0 * offset.y),
		vec2( 3.0 * offset.x, 2.0 * offset.y),

		vec2(-3.0 * offset.x, 3.0 * offset.y),
		vec2(-2.0 * offset.x, 3.0 * offset.y),
		vec2(-1.0 * offset.x, 3.0 * offset.y),
		vec2( 0.0           , 3.0 * offset.y),
		vec2( 1.0 * offset.x, 3.0 * offset.y),
		vec2( 2.0 * offset.x, 3.0 * offset.y),
		vec2( 3.0 * offset.x, 3.0 * offset.y)
	);

	float kernel[49] = float[](
		1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0,
		2.0, 3.0, 4.0, 5.0, 4.0, 3.0, 2.0,
		3.0, 4.0, 5.0, 6.0, 5.0, 4.0, 3.0,
		4.0, 5.0, 6.0, 7.0, 6.0, 5.0, 4.0,
		3.0, 4.0, 5.0, 6.0, 5.0, 4.0, 3.0,
		2.0, 3.0, 4.0, 5.0, 4.0, 3.0, 2.0,
		1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0
	);
	float sum = 0.0;
	for (int i = 0; i < 49; i++) {
		sum += kernel[i];
	}

	float alpha = 0.0;
	for (int i = 0; i < 49; i++) {
		alpha += texture(tex, tex_coord.st + offsets[i]).r * kernel[i];
	}
	alpha /= 15.0 * sum;

	out_color = vec4(0.0, 0.0, 0.0, alpha);
}

