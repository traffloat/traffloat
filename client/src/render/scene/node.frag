uniform sampler2D u_tex;
// Set to 0.5 if targeted by cursor, 1.0 otherwise.
uniform lowp float u_inv_gain;

varying mediump vec2 v_tex_pos;
varying lowp float v_light;

void main() {
    mediump vec4 light_vec = vec4(v_light, v_light, v_light, 1.0);
    mediump vec4 tex_color = texture2D(u_tex, v_tex_pos);
    mediump vec4 output = light_vec * tex_color;
    gl_FragColor = 1.0 - (1.0 - output) * u_inv_gain;
}
