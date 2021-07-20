// Set to 0.5 if targeted by cursor, 1.0 otherwise.
uniform lowp float u_inv_gain;

varying lowp vec4 v_color;

void main() {
    gl_FragColor = 1.0 - (1.0 - v_color) * u_inv_gain;
}
