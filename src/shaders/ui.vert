#version 300 es

in vec3 Vertex_Position;
in vec3 Vertex_Normal;
in vec2 Vertex_Uv;

out vec2 v_Uv;

layout(std140) uniform Camera {
    mat4 ViewProj;
};

layout(std140) uniform Transform {  // set = 1, binding = 0
    mat4 Object;
};
layout(std140) uniform Node_size {  // set = 1, binding = 1
    vec2 NodeSize;
};

void main() {
    v_Uv = Vertex_Uv;
    vec3 position = Vertex_Position * vec3(NodeSize, 0.0);
    gl_Position = ViewProj * Object * vec4(position, 1.0);
}