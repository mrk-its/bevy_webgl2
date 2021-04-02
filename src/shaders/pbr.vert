#version 300 es

precision highp float;

in vec3 Vertex_Position;
in vec3 Vertex_Normal;
in vec2 Vertex_Uv;

#ifdef STANDARDMATERIAL_NORMAL_MAP
in vec4 Vertex_Tangent;
#endif

out vec3 v_WorldPosition;
out vec3 v_WorldNormal;
out vec2 v_Uv;

#ifdef STANDARDMATERIAL_NORMAL_MAP
out vec4 v_WorldTangent;
#endif


layout(std140) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(std140) uniform Transform { // set = 2,  binding = 0
    mat4 Model;
};

void main() {
    vec4 world_position = Model * vec4(Vertex_Position, 1.0);
    v_WorldPosition = world_position.xyz;
    v_WorldNormal = mat3(Model) * Vertex_Normal;
    v_Uv = Vertex_Uv;
#ifdef STANDARDMATERIAL_NORMAL_MAP
    v_WorldTangent = vec4(mat3(Model) * Vertex_Tangent.xyz, Vertex_Tangent.w);
#endif
    gl_Position = ViewProj * world_position;
}