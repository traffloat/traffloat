attribute mediump vec3 a_pos;

uniform mediump mat4 u_trans;

varying lowp float v_height;

void main() {
    gl_Position = u_trans * vec4(a_pos, 1.0);

    if(a_pos.x < 0.8) {
        v_height = a_pos.x / 2.0;
    } else {
        v_height = (a_pos.x - 0.8) * 3.0 + 0.4;
    }
    v_height = v_height * 0.5 + 0.5;
}
