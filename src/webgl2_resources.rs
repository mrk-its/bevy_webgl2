use crate::{
    gl_call,
    renderer::{
        gl_vertex_format, WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram,
        WebGlShader, WebGlTexture, WebGlVertexArrayObject,
    },
};
use bevy::asset::{Handle, HandleUntyped};
use bevy::render::{
    pipeline::{
        BindGroupDescriptor, BindGroupDescriptorId, ColorTargetState, DepthStencilState,
        IndexFormat, InputStepMode, PipelineDescriptor, PrimitiveState, VertexAttribute,
        VertexBufferLayout,
    },
    renderer::{BindGroupId, BufferId, BufferInfo, RenderResourceId, SamplerId, TextureId},
    shader::ShaderStages,
    texture::TextureDescriptor,
};
use bevy::utils::HashMap;
use parking_lot::RwLock;
use std::ops::Range;
use std::{borrow::Cow, sync::Arc};

pub struct GlVertexFormat {
    pub format: u32,
    pub nr_of_components: i32,
    pub normalized: bool,
}

pub struct GlVertexAttribute {
    pub name: Cow<'static, str>,
    pub offset: i32,
    pub format: GlVertexFormat,
    pub attrib_location: i32,
}

impl GlVertexAttribute {
    pub fn from(
        gl: &WebGl2RenderingContext,
        program: &WebGlProgram,
        attr: &VertexAttribute,
    ) -> GlVertexAttribute {
        let attrib_location = gl_call!(gl.get_attrib_location(&program, &*attr.name));
        GlVertexAttribute {
            name: attr.name.to_owned(),
            offset: attr.offset as i32,
            format: gl_vertex_format(&attr.format),
            attrib_location,
        }
    }
}

pub struct GlVertexBufferDescripror {
    pub name: Cow<'static, str>,
    pub stride: i32,
    pub step_mode: InputStepMode,
    pub attributes: Vec<GlVertexAttribute>,
}

impl GlVertexBufferDescripror {
    pub fn from(
        gl: &WebGl2RenderingContext,
        program: &WebGlProgram,
        vertex_buffer_descriptor: &VertexBufferLayout,
    ) -> GlVertexBufferDescripror {
        GlVertexBufferDescripror {
            name: vertex_buffer_descriptor.name.to_owned(),
            stride: vertex_buffer_descriptor.stride as i32,
            step_mode: vertex_buffer_descriptor.step_mode,
            attributes: vertex_buffer_descriptor
                .attributes
                .iter()
                .map(|attr| GlVertexAttribute::from(gl, program, attr))
                .collect(),
        }
    }
}

pub struct WebGL2Pipeline {
    pub shader_stages: ShaderStages,
    pub vertex_buffer_descriptors: Vec<GlVertexBufferDescripror>,
    pub vao: WebGlVertexArrayObject,
    pub vertex_buffer: Option<BufferId>,
    pub index_buffer: Option<BufferId>,
    pub index_format: IndexFormat,
    pub update_vao: bool,
    pub color_target_states: Vec<ColorTargetState>,
    pub depth_stencil: Option<DepthStencilState>,
    pub primitive: PrimitiveState,
    pub scissors_state: Option<ScissorsState>,
}

#[derive(Clone)]
pub struct ScissorsState {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Debug)]
pub enum WebGL2RenderResourceBinding {
    Buffer {
        binding_point: u32,
        buffer: BufferId,
        range: Range<u64>,
    },
    Texture {
        texture_unit: u32,
        texture: TextureId,
    },
    Sampler(SamplerId),
}

#[derive(Debug)]
pub enum Buffer {
    WebGlBuffer(WebGlBuffer),
    Data(Vec<u8>),
}

#[derive(Debug)]
pub struct GlBufferInfo {
    pub buffer: Buffer,
    pub info: BufferInfo,
}

pub struct GlShader {
    pub shader: WebGlShader,
    pub bind_groups: GlBindGroups,
}

impl GlShader {
    pub fn new(shader: WebGlShader, bind_groups: GlBindGroups) -> Self {
        Self {
            shader,
            bind_groups,
        }
    }
}

pub type GlBindGroups = HashMap<String, (u32, u32)>;

pub struct GlProgram {
    pub program: WebGlProgram,
    pub bind_groups: GlBindGroups,
}

impl GlProgram {
    pub fn new(program: WebGlProgram, bind_groups: GlBindGroups) -> Self {
        Self {
            program,
            bind_groups,
        }
    }
}

#[derive(Default, Clone)]
pub struct WebGL2Resources {
    pub binding_point_seq: Arc<RwLock<u32>>,
    pub texture_unit_seq: Arc<RwLock<u32>>,
    pub window_size: Arc<RwLock<(u32, u32)>>,
    pub programs: Arc<RwLock<HashMap<ShaderStages, GlProgram>>>,
    pub binding_points: Arc<RwLock<HashMap<(u32, u32), u32>>>,
    pub texture_units: Arc<RwLock<HashMap<(u32, u32), u32>>>,
    pub bind_groups: Arc<RwLock<HashMap<BindGroupId, Vec<WebGL2RenderResourceBinding>>>>,
    pub buffers: Arc<RwLock<HashMap<BufferId, GlBufferInfo>>>,
    pub texture_descriptors: Arc<RwLock<HashMap<TextureId, TextureDescriptor>>>,
    pub textures: Arc<RwLock<HashMap<TextureId, WebGlTexture>>>,
    pub asset_resources: Arc<RwLock<HashMap<(HandleUntyped, u64), RenderResourceId>>>,
    pub bind_group_layouts: Arc<RwLock<HashMap<BindGroupDescriptorId, BindGroupDescriptor>>>,
    pub pipelines: Arc<RwLock<HashMap<Handle<PipelineDescriptor>, WebGL2Pipeline>>>,
    pub short_buffer_id_seq: Arc<RwLock<u32>>,
    pub short_buffer_ids: Arc<RwLock<HashMap<BufferId, u32>>>,
    pub framebuffers: Arc<RwLock<HashMap<TextureId, WebGlFramebuffer>>>,
    // pub fence_sync: Arc<RwLock<Option<WebGlSync>>>,
}

impl WebGL2Resources {
    fn _get_or_create<T>(&self, storage: &mut HashMap<T, u32>, seq: &mut u32, key: T) -> u32
    where
        T: std::cmp::Eq + std::hash::Hash,
    {
        *storage.entry(key).or_insert_with(|| {
            let ret = *seq;
            *seq += 1;
            ret
        })
    }

    pub fn get_or_create_binding_point(&self, group_index: u32, index: u32) -> u32 {
        let mut storage = self.binding_points.write();
        let mut seq = self.binding_point_seq.write();
        self._get_or_create(&mut *storage, &mut seq, (group_index, index))
    }

    pub fn get_or_create_texture_unit(&self, group_index: u32, index: u32) -> u32 {
        let mut storage = self.texture_units.write();
        let mut seq = self.texture_unit_seq.write();

        self._get_or_create(&mut *storage, &mut seq, (group_index, index))
        // adding 1 to return value here should force using next texture unit
        // but it seems to not work
    }

    pub fn short_buffer_id(&self, buffer_id: BufferId) -> u32 {
        let mut storage = self.short_buffer_ids.write();
        let mut seq = self.short_buffer_id_seq.write();
        self._get_or_create(&mut storage, &mut seq, buffer_id)
    }
}
