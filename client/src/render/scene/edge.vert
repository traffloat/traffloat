// Vertex position on the unit cylinder,
// where x^2 + y^2 = 1 and z = 0 or 1.
attribute mediump vec3 a_pos;
// Unit normal of the vertex.
// This should be normal to the Z-axis.
attribute mediump vec3 a_normal;

// Transformation from unit cylinder coordinates to camera coordinates
uniform mediump mat4 u_trans;
// Sun direction relative from the camera.
// A unit vector.
uniform mediump vec3 u_trans_sun;
// Base RGBA of the corridor
uniform lowp vec4 u_color;
// Weight of ambient illumination component
uniform lowp float u_ambient;
// Weight of diffuse illumination component
uniform lowp float u_diffuse;
// Weight of specular illumination component
uniform lowp float u_specular;
// Specular illumination coefficient
uniform mediump float u_specular_coef;

varying lowp vec4 v_color;

void main() {
    gl_Position = u_trans * vec4(a_pos, 1.0);

    mediump mat3 trans = mat3(u_trans);
    lowp float diffuse = u_diffuse * max(0.0, dot(trans * a_normal, u_trans_sun));
    mediump vec3 halfway = normalize(u_trans_sun + vec3(0.0, 0.0, -1.0));
    lowp float specular = u_specular * max(0.0, pow(dot(trans * a_normal, halfway), u_specular_coef));

    v_color.rgb = u_color.rgb * min(1.0, u_ambient + diffuse + specular);
    v_color.a = u_color.a;
}
