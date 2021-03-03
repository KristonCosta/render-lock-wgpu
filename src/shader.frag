#version 450

layout(location=0) in vec2 v_tex_coords;
layout(location=1) in vec3 v_normal;
layout(location=2) in vec3 v_position;

layout(location=0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

layout(set = 0, binding = 2) uniform texture2D t_normal;
layout(set = 0, binding = 3) uniform sampler s_normal;


layout(set=1, binding=0)
uniform Uniforms {
    vec3 u_view_position;
    mat4 u_view_proj;
};

layout(set=2, binding=0) uniform Light {
    vec3 light_position;
    vec3 light_color;
};

void main() {

    vec4 object_color = texture(sampler2D(t_diffuse, s_diffuse), v_tex_coords);
    // Not supporting normal mapping for now
    // vec4 object_normal = texture(sampler2D(t_normal, s_normal), v_tex_coords);

    float ambient_strength = 0.1;
    vec3 ambient_color = light_color * ambient_strength;
    vec3 normal = normalize(v_normal);
    // vec3 normal = normalize(object_normal.rgb * 2.0 - 1.0);
    vec3 light_dir = normalize(light_position - v_position);

    vec3 view_dir = normalize(u_view_position - v_position);
    vec3 half_dir = normalize(view_dir + light_dir);
    vec3 reflect_dir = reflect(-light_dir, normal);

    float specular_strength = pow(max(dot(normal, half_dir), 0.0), 32);
    vec3 specular_color = specular_strength * light_color;

    float diffuse_strength = max(dot(normal, light_dir), 0.0);
    vec3 diffuse_color = light_color * diffuse_strength;

    vec3 result = (ambient_color + diffuse_color + specular_color) * object_color.xyz;

    f_color = vec4(result, object_color.a);
}