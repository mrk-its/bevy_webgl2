#version 300 es

precision highp float;

in vec2 v_Uv;

out vec4 o_Target;

layout(std140) uniform ColorMaterial_color {  // set = 1, binding = 0
    vec4 Color;
};

# ifdef COLORMATERIAL_TEXTURE
uniform sampler2D ColorMaterial_texture;  // set = 1, binding = 1
# endif

vec4 encodeSRGB(vec4 linearRGB_in)
{
    vec3 linearRGB = linearRGB_in.rgb;
    vec3 a = 12.92 * linearRGB;
    vec3 b = 1.055 * pow(linearRGB, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linearRGB);
    return vec4(mix(a, b, c), linearRGB_in.a);
}

void main() {
    vec4 color = Color;
# ifdef COLORMATERIAL_TEXTURE
    color *= texture(
        ColorMaterial_texture,
        v_Uv
    );
# endif
    o_Target = encodeSRGB(color);
}
