attribute vec3 a_vertex_pos;

uniform mat4 u_projection;
uniform mat4 u_object;

void main() {
	gl_Position = u_projection * u_object * vec4(a_vertex_pos, 1.) * 0.1;
}
