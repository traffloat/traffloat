attribute mediump vec3 a_pos;
attribute mediump vec3 a_normal;
attribute mediump vec2 a_tex_pos;
attribute mediump vec4 a_tex_offset;

uniform mediump mat4 u_proj;
uniform mediump vec3 u_sun;
uniform mediump float u_brightness;
uniform mediump vec2 u_tex_dim;

varying mediump vec2 v_tex_pos;
varying lowp float v_light;

void main() {
    gl_Position = u_proj * vec4(a_pos, 1.0);

    v_tex_pos.x = a_tex_offset[0] + a_tex_pos.x * a_tex_offset[1];
    v_tex_pos.y = a_tex_offset[2] + a_tex_pos.y * a_tex_offset[3];

    v_tex_pos /= u_tex_dim;

    mediump float sun_magnitude = dot(u_sun, a_normal);
    sun_magnitude = max(0., min(1., sun_magnitude)) * 0.5 + 0.5;
    v_light = sun_magnitude * u_brightness;
}
