#version 450

layout(location=0) in vec3 a_position;
layout(location=1) in vec2 a_tex_coords;
layout(location=5) in vec4 model_matrix0;
layout(location=6) in vec4 model_matrix1;
layout(location=7) in vec4 model_matrix2;
layout(location=8) in vec4 model_matrix3;

layout(location=0) out vec2 v_tex_coords;


layout(set=1, binding=0) uniform Uniforms {
    mat4 u_view_proj;
};

void main() {
    mat4 model_matrix = mat4(model_matrix0, model_matrix1, model_matrix2, model_matrix3);
    v_tex_coords = a_tex_coords;
    gl_Position = u_view_proj * model_matrix * vec4(a_position, 1.0);
}