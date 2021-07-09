uniform mediump vec3 u_color;

varying lowp float v_height;

void main() {
    gl_FragColor = vec4(u_color * v_height, 1.0);
}
