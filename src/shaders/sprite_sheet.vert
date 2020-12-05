#version 300 es

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

out vec2 v_Uv;
out vec4 v_Color;

layout(std140) uniform Camera {
    mat4 ViewProj;
};

// TODO: merge dimensions into "sprites" buffer when that is supported in the Uniforms derive abstraction
layout(std140) uniform TextureAtlas_size {  // set = 1, binding = 0
    vec2 AtlasSize;
};

struct Rect {
    vec2 begin;
    vec2 end;
};

layout(std140) uniform TextureAtlas_textures {  // set = 1, binding = 1
    Rect[256] Textures;
};


layout(std140) uniform Transform {  // set = 2, binding = 0
    mat4 SpriteTransform;
};

layout(std140) uniform TextureAtlasSprite { // set = 2, binding = 1
    vec4 TextureAtlasSprite_color;
    uint TextureAtlasSprite_index;
};

void main() {
    Rect sprite_rect = Textures[TextureAtlasSprite_index];
    vec2 sprite_dimensions = sprite_rect.end - sprite_rect.begin;
    vec3 vertex_position = vec3(Vertex_Position.xy * sprite_dimensions, 0.0);
    vec2 atlas_positions[4] = vec2[](
        vec2(sprite_rect.begin.x, sprite_rect.end.y),
        sprite_rect.begin,
        vec2(sprite_rect.end.x, sprite_rect.begin.y),
        sprite_rect.end
    );
    v_Uv = atlas_positions[gl_VertexID] / AtlasSize;
    v_Color = TextureAtlasSprite_color;
    gl_Position = ViewProj * SpriteTransform * vec4(ceil(vertex_position), 1.0);
}
