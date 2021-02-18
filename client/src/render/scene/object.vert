attribute highp vec3 a_pos;
attribute highp vec2 a_tex_pos;
attribute highp vec3 a_normal;

uniform highp mat4 u_proj;
uniform highp vec3 u_sun;

varying highp vec2 v_tex_pos;
varying mediump float v_light;

void main() {
    gl_Position = u_proj * vec4(a_pos, 1.0);
    v_tex_pos = a_tex_pos;
    v_light = dot(u_sun, a_normal);
}
