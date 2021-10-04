#version 450

layout (location = 0) in vec2 position;

out TVertexData {
	float light;
	vec2 uv;
} outData;

uniform mat4 view;
uniform mat4 proj;
uniform float height;
uniform float light;

void main() {
    vec4 newPos = vec4(-position.x,  height, position.y, 1.0);
	gl_Position = proj * view * newPos;
    outData.light = light;
	outData.uv.x = position.x / 64.0;
	outData.uv.y = position.y / 64.0;
}