varying mediump vec3 v_pos;
varying mediump vec3 v_normal;
varying lowp vec3 v_color;
varying mediump float v_shininess;
varying lowp vec3 v_reflect;

uniform mediump vec3 u_sun;
uniform mediump vec3 u_camera;

void main() {
	vec3 source = normalize(u_sun - v_pos);
	vec3 view = normalize(u_camera - v_pos);
	vec3 reflected = reflect(source, v_normal);
	float diffuse = max(0.0, dot(source, v_normal));
	float specular = pow(dot(reflected, view), v_shininess);

	vec3 comps = vec3(diffuse, specular, 1.0);
	float mult = dot(comps, v_reflect);

	gl_FragColor = vec4(v_color * mult, 1.0);
}
