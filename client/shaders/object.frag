precision lowp float;

varying lowp vec3 v_color;
varying lowp vec3 v_normal;
varying lowp vec3 v_pos;

uniform vec3 u_sun;
uniform vec3 u_camera;
uniform float u_shininess;

uniform vec3 u_comp;

void main() {
	vec3 source = normalize(u_sun - v_pos);
	vec3 view = normalize(u_camera - v_pos);
	vec3 reflected = reflect(source, v_normal);
	float diffuse = max(0.0, dot(source, v_normal));
	float specular = pow(dot(reflected, view), u_shininess);

	vec3 comps = vec3(diffuse, specular, 1.0);
	float mult = dot(comps, u_comp);

	gl_FragColor = vec4(v_color * mult, 1.0);
}
