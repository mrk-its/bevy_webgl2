#version 300 es

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

out vec2 v_Uv;
out vec4 v_Color;

layout(std140) uniform CameraViewProj {
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
    vec4 color;
    uint index;
    uint flip;
};

void main() {
    Rect sprite_rect = Textures[index];
    vec2 sprite_dimensions = sprite_rect.end - sprite_rect.begin;
    vec3 vertex_position = vec3(Vertex_Position.xy * sprite_dimensions, 0.0);

    // Specify the corners of the sprite
    vec2 bottom_left = vec2(sprite_rect.begin.x, sprite_rect.end.y);
    vec2 top_left = sprite_rect.begin;
    vec2 top_right = vec2(sprite_rect.end.x, sprite_rect.begin.y);
    vec2 bottom_right = sprite_rect.end;

    // Flip the sprite if necessary
    uint x_flip_bit = 1u;
    uint y_flip_bit = 2u;

    vec2 tmp;
    if ((flip & x_flip_bit) == x_flip_bit) {
        // Shuffle the corners to flip around x
        tmp = bottom_left;
        bottom_left = bottom_right;
        bottom_right = tmp;
        tmp = top_left;
        top_left = top_right;
        top_right = tmp;
    }
    if ((flip & y_flip_bit) == y_flip_bit) {
        // Shuffle the corners to flip around y
        tmp = bottom_left;
        bottom_left = top_left;
        top_left = tmp;
        tmp = bottom_right;
        bottom_right = top_right;
        top_right = tmp;
    }

    vec2 atlas_positions[4] = vec2[](
        bottom_left,
        top_left,
        top_right,
        bottom_right
    );
    v_Uv = atlas_positions[gl_VertexID] / AtlasSize;
    v_Color = color;
    gl_Position = ViewProj * SpriteTransform * vec4(ceil(vertex_position), 1.0);
}
