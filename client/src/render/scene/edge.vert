attribute mediump vec3 a_pos;
attribute mediump vec3 a_normal;

uniform mediump mat4 u_trans;
uniform mediump vec3 u_trans_sun;
uniform lowp vec4 u_color;
uniform lowp float u_ambient;
uniform lowp float u_diffuse;
uniform lowp float u_specular;
uniform uint u_specular_coef;

varying lowp vec4 v_color;

void main() {
    gl_Position = u_trans * vec4(a_pos, 1.0);

    mediump mat3 trans = mat3(u_trans);
    lowp float diffuse = dot(trans * a_normal, u_trans_sun);
    mediump vec3 halfway = normalize(u_trans_sun + vec3(0.0, 0.0, -1.0));
    lowp float specular = pow(dot(trans * a_normal, halfway), u_specular_coef);
    v_color = u_color * min(1.0, u_ambient + diffuse + specular);
}
