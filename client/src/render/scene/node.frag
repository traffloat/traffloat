uniform sampler2D u_tex;
// Set to 0.5 if targeted by cursor, 1.0 otherwise.
uniform lowp float u_inv_gain;

uniform bool u_uses_texture;

varying mediump vec2 v_tex_pos;
varying lowp vec3 v_filter;

void main() {
    mediump vec4 filter_vec = vec4(v_filter, 1.0);
    mediump vec4 tex_color = u_uses_texture ? texture2D(u_tex, v_tex_pos) : vec4(1.0, 1.0, 1.0, 1.0);
    mediump vec4 base = filter_vec * tex_color;
    gl_FragColor = 1.0 - (1.0 - base) * u_inv_gain;
}
