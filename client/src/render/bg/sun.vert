uniform mediump mat4 u_sun_mat;
uniform mediump float u_body_radius;
uniform mediump float u_aura_radius;
uniform mediump float u_aspect;

attribute mediump vec2 a_pos;

varying mediump vec2 v_pos;

void main() {
    mediump vec2 screen_pos = vec2(a_pos.x / u_aspect, a_pos.y);
    screen_pos *= u_body_radius + u_aura_radius;
    gl_Position = u_sun_mat * vec4(screen_pos, 0.5, 1.0);
    v_pos = a_pos;
}
