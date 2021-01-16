attribute vec3 a_vertex_pos;
attribute vec3 a_vertex_color;

uniform mat4 u_projection;

varying lowp vec3 v_color;

void main() {
	gl_Position = u_projection * vec4(a_vertex_pos, 1.) * 0.1;
	v_color = a_vertex_color;
}
