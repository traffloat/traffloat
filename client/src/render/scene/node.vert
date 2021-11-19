attribute mediump vec3 a_pos;
attribute mediump vec3 a_normal;
attribute mediump vec2 a_tex_pos;

uniform mediump mat4 u_proj;
uniform mediump vec3 u_sun;
uniform mediump vec3 u_filter;
uniform mediump vec2 u_tex_dim;

varying mediump vec2 v_tex_pos;
varying lowp vec3 v_filter;

void main() {
    gl_Position = u_proj * vec4(a_pos, 1.0);

    v_tex_pos = a_tex_pos;

    mediump float sun_magnitude = dot(u_sun, a_normal);
    sun_magnitude = max(0., min(1., sun_magnitude)) * 0.5 + 0.5;

    v_filter = sun_magnitude * u_filter;
}
