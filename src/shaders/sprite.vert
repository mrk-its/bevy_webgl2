#version 300 es

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

out vec2 v_Uv;

layout(std140) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(std140) uniform Transform {  // set = 2, binding = 0
    mat4 Model;
};

layout(std140) uniform Sprite {  // set = 2, binding = 1
    vec2 size;
    uint flip;
};

void main() {
    vec2 uv = Vertex_Uv;

    // Flip the sprite if necessary by flipping the UVs

    uint x_flip_bit = 1u; // The X flip bit
    uint y_flip_bit = 2u; // The Y flip bit

    // Note: Here we subtract f32::EPSILON from the flipped UV coord. This is due to reasons unknown
    // to me (@zicklag ) that causes the uv's to be slightly offset and causes over/under running of
    // the sprite UV sampling which is visible when resizing the screen.
    float epsilon = 0.00000011920929;
    if ((flip & x_flip_bit) == x_flip_bit) {
        uv = vec2(1.0 - uv.x - epsilon, uv.y);
    }
    if ((flip & y_flip_bit) == y_flip_bit) {
        uv = vec2(uv.x, 1.0 - uv.y - epsilon);
    }

    v_Uv = uv;

    vec3 position = Vertex_Position * vec3(size, 1.0);
    gl_Position = ViewProj * Model * vec4(position, 1.0);
}
