varying lowp vec4 v_color;

void main() {
    gl_FragColor = v_color;
    // gl_FragColor = v_color * 0.00001 + vec4(1., 1., 1., 1.);
}
