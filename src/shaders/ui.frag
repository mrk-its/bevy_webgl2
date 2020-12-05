#version 300 es

precision highp float;
vec4 encodeColor(vec4 linearRGB_in)
{
    vec3 linearRGB = linearRGB_in.rgb;
    vec3 a = 12.92 * linearRGB;
    vec3 b = 1.055 * pow(linearRGB, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linearRGB);
    return vec4(mix(a, b, c), linearRGB_in.a);
}

# define TEXTURE_2D sampler2D
# define sampler2D(a, b) (a)
# define gl_VertexIndex gl_VertexID

in vec2 v_Uv;
out vec4 o_Target;

layout(std140) uniform ColorMaterial_color {  // set = 2, binding = 0
    vec4 Color;
};

# ifdef COLORMATERIAL_TEXTURE
uniform TEXTURE_2D ColorMaterial_texture;  // set = 2, binding = 1
# endif

void main() {
    vec4 color = Color;
# ifdef COLORMATERIAL_TEXTURE
    color *= texture(
        sampler2D(ColorMaterial_texture, ColorMaterial_texture_sampler),
        v_Uv);
# endif
    o_Target = encodeColor(color);
}
