#version 450

in TVertexData {
	float light;
	vec3 uv;
} inData;

out vec4 fragColor;

uniform sampler2DArray image;
uniform int sky;

void main() {
	if (sky == 1) {
		fragColor = vec4( gl_FragCoord.x/1680.0, gl_FragCoord.y/1050.0, 0.0, 1.0 );
	} else {
		vec4 color = texture(image, inData.uv);
		if (color.w < 0.01) {
			discard;
		}
		fragColor = vec4(inData.light * vec3( color.x, color.y, color.z ), color.w);	
	}
}