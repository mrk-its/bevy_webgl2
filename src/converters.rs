use crate::renderer::*;
use bevy::{prelude::warn, render::texture::TextureFormat};

pub trait WebGl2From<T> {
    fn from(val: T) -> Self;
}

pub trait WebGl2Into<U> {
    fn webgl2_into(self) -> U;
}

impl<T, U> WebGl2Into<U> for T
where
    U: WebGl2From<T>,
{
    fn webgl2_into(self) -> U {
        U::from(self)
    }
}

impl WebGl2From<TextureFormat> for (u32, u32, u32) {
    fn from(val: TextureFormat) -> Self {
        match val {
            TextureFormat::R8Unorm => (Gl::R8, Gl::RED, Gl::UNSIGNED_BYTE),
            TextureFormat::R8Snorm => (Gl::R8_SNORM, Gl::RED, Gl::BYTE),
            TextureFormat::R8Uint => (Gl::R8UI, Gl::RED_INTEGER, Gl::UNSIGNED_BYTE),
            TextureFormat::R8Sint => (Gl::R8I, Gl::RGBA_INTEGER, Gl::INT),
            TextureFormat::R16Uint => (Gl::R16UI, Gl::RGBA_INTEGER, Gl::UNSIGNED_INT),
            TextureFormat::R16Sint => (Gl::R16I, Gl::RGBA_INTEGER, Gl::INT),
            TextureFormat::R16Float => (Gl::R16F, Gl::RED, Gl::HALF_FLOAT),
            TextureFormat::Rg8Unorm => (Gl::RG8, Gl::RG, Gl::UNSIGNED_BYTE),
            TextureFormat::Rg8Snorm => (Gl::RG8_SNORM, Gl::RG, Gl::BYTE),
            TextureFormat::Rg8Uint => (Gl::RG8UI, Gl::RG_INTEGER, Gl::UNSIGNED_BYTE),
            TextureFormat::Rg8Sint => (Gl::RG8I, Gl::RG_INTEGER, Gl::BYTE),
            TextureFormat::R32Uint => (Gl::R32UI, Gl::RED_INTEGER, Gl::UNSIGNED_INT),
            TextureFormat::R32Sint => (Gl::R32I, Gl::RED_INTEGER, Gl::INT),
            TextureFormat::R32Float => (Gl::R32F, Gl::RED, Gl::FLOAT),
            TextureFormat::Rg16Uint => (Gl::RG16UI, Gl::RG_INTEGER, Gl::UNSIGNED_SHORT),
            TextureFormat::Rg16Sint => (Gl::RG16I, Gl::RG_INTEGER, Gl::SHORT),
            TextureFormat::Rg16Float => (Gl::RG16F, Gl::RG, Gl::HALF_FLOAT),
            TextureFormat::Rgba8Unorm => (Gl::RGBA8, Gl::RGBA, Gl::UNSIGNED_BYTE),
            TextureFormat::Rgba8UnormSrgb => (Gl::SRGB8_ALPHA8, Gl::RGBA, Gl::UNSIGNED_BYTE),
            TextureFormat::Rgba8Snorm => (Gl::RGBA8_SNORM, Gl::RGBA, Gl::BYTE),
            TextureFormat::Rgba8Uint => (Gl::RGBA8UI, Gl::RGBA_INTEGER, Gl::UNSIGNED_BYTE),
            TextureFormat::Rgba8Sint => (Gl::RGBA8I, Gl::RGBA_INTEGER, Gl::BYTE),
            TextureFormat::Bgra8Unorm => {
                warn!("Bgra8Unorm is unsupported, using Rgba8Unorm instead");
                (Gl::RGBA8, Gl::RGBA, Gl::UNSIGNED_BYTE)
            }
            TextureFormat::Bgra8UnormSrgb => {
                warn!("Bgra8UnormSrgb is unsupported, using Rgba8UnormSrgb instead");
                (Gl::SRGB8_ALPHA8, Gl::RGBA, Gl::UNSIGNED_BYTE)
            }
            TextureFormat::Rgb10a2Unorm => {
                (Gl::RGB10_A2, Gl::RGBA, Gl::UNSIGNED_INT_2_10_10_10_REV)
            }
            TextureFormat::Rg11b10Float => (
                Gl::R11F_G11F_B10F,
                Gl::RGB,
                Gl::UNSIGNED_INT_10F_11F_11F_REV,
            ),
            TextureFormat::Rg32Uint => (Gl::RG32UI, Gl::RG_INTEGER, Gl::UNSIGNED_INT),
            TextureFormat::Rg32Sint => (Gl::RG32I, Gl::RG_INTEGER, Gl::INT),
            TextureFormat::Rg32Float => (Gl::RG32F, Gl::RG, Gl::FLOAT),
            TextureFormat::Rgba16Uint => (Gl::RGBA16UI, Gl::RGBA_INTEGER, Gl::UNSIGNED_SHORT),
            TextureFormat::Rgba16Sint => (Gl::RGBA16I, Gl::RGBA_INTEGER, Gl::SHORT),
            TextureFormat::Rgba16Float => (Gl::RGBA16F, Gl::RGBA, Gl::HALF_FLOAT),
            TextureFormat::Rgba32Uint => (Gl::RGBA32UI, Gl::RGBA_INTEGER, Gl::UNSIGNED_INT),
            TextureFormat::Rgba32Sint => (Gl::RGBA32I, Gl::RGBA_INTEGER, Gl::INT),
            TextureFormat::Rgba32Float => (Gl::RGBA32F, Gl::RGBA, Gl::FLOAT),
            TextureFormat::Depth32Float => (Gl::DEPTH_COMPONENT32F, Gl::DEPTH_COMPONENT, Gl::FLOAT),
            TextureFormat::Depth24Plus => (
                Gl::DEPTH24_STENCIL8,
                Gl::DEPTH_STENCIL,
                Gl::UNSIGNED_INT_24_8,
            ),
            TextureFormat::Depth24PlusStencil8 => (
                Gl::DEPTH24_STENCIL8,
                Gl::DEPTH_STENCIL,
                Gl::UNSIGNED_INT_24_8,
            ),
        }
    }
}
