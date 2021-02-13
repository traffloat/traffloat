uniform lowp vec3 u_color;

uniform mediump float u_body_radius;
uniform mediump float u_aura_radius;

varying mediump vec2 v_pos;

void main() {
    mediump float body_ratio = u_body_radius / (u_body_radius + u_aura_radius);
    mediump float dist = sqrt(v_pos.x * v_pos.x + v_pos.y * v_pos.y);
    mediump float radius = max(min(dist, 1.0), body_ratio);
    mediump float intensity = (1.0 - radius) / (1.0 - body_ratio);
    lowp vec3 color = u_color * intensity;
    gl_FragColor = vec4(color.x, color.y, color.z, 1.0);
}
