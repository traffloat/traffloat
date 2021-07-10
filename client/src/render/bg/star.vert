attribute mediump vec3 a_pos;

// Linear transformation matrix from world coordinates to clip space.
// This is mat3 instead of mat4 because translation should not be performed.
uniform mediump mat3 u_trans;

void main() {
    gl_Position = vec4(u_trans * a_pos, 1.0);
}
