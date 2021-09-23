#version 450

in TVertexData {
	float light;
	vec2 uv;
} inData;

out vec4 fragColor;

uniform sampler2D image;

void main() {
	vec4 color = texture(image, inData.uv);
	fragColor = vec4(inData.light * vec3( color.x, color.y, color.z ), color.w);	
}