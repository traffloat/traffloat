uniform mediump vec3 u_screen_pos;
uniform mediump float u_body_radius;
uniform mediump float u_aura_radius;
uniform mediump float u_aspect;

attribute mediump vec2 a_pos;

varying mediump vec2 v_pos;

void main() {
    mediump vec2 pos = vec2(u_screen_pos.x, u_screen_pos.y);
    pos += a_pos * (u_body_radius + u_aura_radius);
    mediump vec4 homo = vec4(pos.x / u_aspect, pos.y, 0.5, 1.0);
    gl_Position = homo;
    v_pos = a_pos;
}
