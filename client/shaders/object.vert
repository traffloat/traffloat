attribute mediump vec3 a_vertex_pos;
attribute mediump vec3 a_vertex_normal;
attribute mediump vec3 a_vertex_color;
attribute mediump float a_vertex_shininess;
attribute lowp vec3 a_vertex_reflect;

uniform highp mat4 u_projection;
uniform highp mat4 u_object;

varying mediump vec3 v_pos;
varying mediump vec3 v_normal;
varying lowp vec3 v_color;
varying mediump vec3 v_shininess;
varying lowp vec3 v_reflect;

void main() {
	gl_Position = u_projection * u_object * vec4(a_vertex_pos, 1.) * 0.1;
	v_pos = a_vertex_pos;
	v_normal = a_vertex_normal;
	v_color = a_vertex_color;
	v_shininess = a_vertex_shininess;
	v_reflect = a_vertex_reflect;
}
