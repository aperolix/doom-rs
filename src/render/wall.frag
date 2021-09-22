#version 450

in TVertexData {
	float light;
	vec3 normal;
	vec2 uv;
} inData;

out vec4 fragColor;

uniform sampler2D image;

void main() {
	vec4 color = texture(image, inData.uv);
	float att = dot(inData.normal, inData.normal);//normalize(vec3( 0.5, 0.0, 0.5 )) );
    //fragColor = vec4(inData.light * vec3( mod( inData.uv.x, 1), mod(inData.uv.y, 1), 0.0 ) * abs( att ) * 0.5, color.w);
	fragColor = vec4(inData.light * vec3( color.x, color.y, color.z ) * abs( att ) * 0.5, color.w);	
}