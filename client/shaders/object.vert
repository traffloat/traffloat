attribute vec3 a_vertex_pos;
attribute vec3 a_vertex_normal;
attribute vec3 a_vertex_color;

uniform mat4 u_projection;
uniform mat4 u_object;

varying lowp vec3 v_color;
varying lowp vec3 v_normal;
varying lowp vec3 v_pos;

void main() {
	gl_Position = u_projection * u_object * vec4(a_vertex_pos, 1.) * 0.1;
	v_color = a_vertex_color;
	v_normal = a_vertex_normal;
	v_pos = a_vertex_pos;
}
