uniform sampler2D u_tex;

varying highp vec2 v_tex_pos;
varying mediump float v_light;

void main() {
    mediump vec4 light_vec = vec4(v_light, v_light, v_light, 1.0);
    mediump vec4 tex_color = texture2D(u_tex, v_tex_pos);
    gl_FragColor = tex_color * light_vec;
}
