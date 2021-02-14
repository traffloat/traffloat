attribute highp vec3 pos;
uniform highp mat4 u_proj;

varying mediump vec3 pos;

void main() {
    gl_Position = u_proj * vec4(pos, 1.0);
    v_pos = pos;
}
