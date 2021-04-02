use super::{compile_shader, link_program, reflect_layout, Gl};
use crate::{
    converters::*, gl_call, Buffer, Device, GlBufferInfo, GlShader, GlVertexBufferDescripror,
    WebGL2Pipeline, WebGL2RenderResourceBinding, WebGL2Resources,
};
use bevy::asset::{Assets, Handle, HandleUntyped};
use bevy::log::prelude::*;
use bevy::render::{
    pipeline::{
        BindGroupDescriptor, BindGroupDescriptorId, BindType, IndexFormat, PipelineDescriptor,
        PipelineLayout,
    },
    renderer::{
        BindGroup, BufferId, BufferInfo, BufferMapMode, BufferUsage, RenderResourceBinding,
        RenderResourceContext, RenderResourceId, SamplerId, TextureId,
    },
    shader::{Shader, ShaderError, ShaderSource, ShaderStage, ShaderStages},
    texture::{SamplerDescriptor, TextureDescriptor},
};
use bevy::utils::HashMap;
use bevy::window::Window;
use parking_lot::RwLock;
use std::{ops::Range, sync::Arc};
#[derive(Clone)]
pub struct WebGL2RenderResourceContext {
    pub device: Arc<Device>,
    pub resources: WebGL2Resources,
    pub pipeline_descriptors: Arc<RwLock<HashMap<Handle<PipelineDescriptor>, PipelineDescriptor>>>,
    pub swapchain_texture: TextureId,
    initialized: bool,
}

unsafe impl Send for WebGL2RenderResourceContext {}
unsafe impl Sync for WebGL2RenderResourceContext {}

pub const BIND_BUFFER_ALIGNMENT: usize = 256;
pub const STORAGE_BUFFER_SIZE: usize = 65536;

impl WebGL2RenderResourceContext {
    pub fn new(device: Arc<crate::Device>) -> Self {
        WebGL2RenderResourceContext {
            device,
            resources: WebGL2Resources::default(),
            pipeline_descriptors: Default::default(),
            initialized: false,
            swapchain_texture: TextureId::new(),
        }
    }

    pub fn add_texture_descriptor(&self, texture: TextureId, descriptor: TextureDescriptor) {
        self.resources
            .texture_descriptors
            .write()
            .insert(texture, descriptor);
    }

    pub fn create_bind_group_layout(&self, descriptor: &BindGroupDescriptor) {
        if self.bind_group_descriptor_exists(descriptor.id) {
            return;
        };
        self.resources
            .bind_group_layouts
            .write()
            .insert(descriptor.id, descriptor.clone());
    }

    pub fn compile_shader(&self, shader: &Shader) -> GlShader {
        let shader_type = match shader.stage {
            ShaderStage::Vertex => Gl::VERTEX_SHADER,
            ShaderStage::Fragment => Gl::FRAGMENT_SHADER,
            ShaderStage::Compute => panic!("compute shaders are not supported!"),
        };

        match &shader.source {
            ShaderSource::Glsl(source) => {
                info!("compiling shader: {:?}", source);
                compile_shader(&self.device.get_context(), shader_type, source).unwrap()
            }
            _ => {
                panic!("unsupported shader format");
            }
        }
    }

    #[allow(unused_variables)]
    pub fn initialize(&mut self, winit_window: &winit::window::Window) {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        let size = winit_window.inner_size();
        let ctx_options = js_sys::Object::new();
        // TODO - test performance for alpha disabled / enabled
        // #[allow(unused_unsafe)]
        // unsafe {
        //     js_sys::Reflect::set(&ctx_options, &"alpha".into(), &false.into()).unwrap();
        // }
        let gl = winit_window
            .canvas()
            .get_context_with_context_options("webgl2", &ctx_options)
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();

        let uniform_buffer_offset_alignment = gl
            .get_parameter(Gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT)
            .unwrap()
            .as_f64()
            .unwrap() as usize;
        info!(
            "uniform_buffer_offset_alignment: {:?}",
            uniform_buffer_offset_alignment
        );

        // let ret = gl`
        //     .get_framebuffer_attachment_parameter(
        //         Gl::FRAMEBUFFER,
        //         Gl::BACK,
        //         Gl::FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING,
        //     )
        //     .unwrap()
        //     .as_f64()
        //     .unwrap() as u32;

        // info!(
        //     "FRAMEBUFFER_ATTACHMENT_COLOR_ENCODING linear: {:?}, srgb: {:?}",
        //     ret == Gl::LINEAR,
        //     ret == Gl::SRGB
        // );

        gl_call!(gl.viewport(0, 0, size.width as i32, size.height as i32));
        gl_call!(gl.enable(Gl::BLEND));
        gl_call!(gl.enable(Gl::DEPTH_TEST));
        self.device.set_context(gl);
        self.initialized = true;
    }
}

impl RenderResourceContext for WebGL2RenderResourceContext {
    fn reflect_pipeline_layout(
        &self,
        shaders: &Assets<Shader>,
        shader_stages: &ShaderStages,
        _enforce_bevy_conventions: bool,
    ) -> PipelineLayout {
        let gl_shaders: Vec<GlShader> = shader_stages
            .iter()
            .map(|handle| self.compile_shader(shaders.get(&handle).unwrap()))
            .collect();

        let program =
            link_program(&*self.device.get_context(), &gl_shaders).expect("WebGL program");

        let gl = &self.device.get_context();

        let layout = reflect_layout(&*gl, &program);
        debug!("reflected layout: {:#?}", layout);
        self.resources
            .programs
            .write()
            .insert(shader_stages.clone(), program);
        layout
    }

    fn get_aligned_texture_size(&self, data_size: usize) -> usize {
        data_size
    }

    fn get_aligned_uniform_size(&self, size: usize, _dynamic: bool) -> usize {
        return (size + BIND_BUFFER_ALIGNMENT - 1) & !(BIND_BUFFER_ALIGNMENT - 1);
    }

    fn create_swap_chain(&self, window: &Window) {
        let gl = &self.device.get_context();
        let mut window_size = self.resources.window_size.write();
        *window_size = (window.physical_width(), window.physical_height());
        gl_call!(gl.viewport(0, 0, window_size.0 as i32, window_size.1 as i32,));
    }

    fn next_swap_chain_texture(&self, window: &Window) -> TextureId {
        let mut window_size = self.resources.window_size.write();
        *window_size = (window.physical_width(), window.physical_height());
        self.swapchain_texture
    }

    fn drop_swap_chain_texture(&self, _render_resource: TextureId) {}

    fn drop_all_swap_chain_textures(&self) {}

    fn create_sampler(&self, _sampler_descriptor: &SamplerDescriptor) -> SamplerId {
        SamplerId::new()
    }

    fn create_texture(&self, texture_descriptor: TextureDescriptor) -> TextureId {
        let texture_id = TextureId::new();
        self.add_texture_descriptor(texture_id, texture_descriptor);
        let gl = &self.device.get_context();
        let texture = gl_call!(gl.create_texture()).unwrap();

        let size = texture_descriptor.size;
        gl_call!(gl.bind_texture(Gl::TEXTURE_2D, Some(&texture)));

        let (internal_format, format, _type) = texture_descriptor.format.webgl2_into();

        gl_call!(
            gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                Gl::TEXTURE_2D,
                0, //destination_mip_level as i32,
                internal_format as i32,
                size.width as i32,
                size.height as i32,
                0,
                format,
                _type,
                None as Option<&[u8]>,
            )
        )
        .unwrap();
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32));
        gl_call!(gl.bind_texture(
            Gl::TEXTURE_2D,
            None as Option<&crate::renderer::WebGlTexture>,
        ));

        self.resources.textures.write().insert(texture_id, texture);
        texture_id
    }

    fn create_buffer(&self, info: BufferInfo) -> BufferId {
        let buffer_id = BufferId::new();
        trace!(
            "create_buffer, info: {:?}, short_id: {:?}, buffer_id: {:?}",
            info,
            self.resources.short_buffer_id(buffer_id),
            buffer_id,
        );

        let buffer = if info
            .buffer_usage
            .contains(BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC)
        {
            // uninitialzied in-memory buffer
            let mut data = Vec::with_capacity(info.size);
            unsafe { data.set_len(info.size) };
            Buffer::Data(data)
        } else {
            let size = if info.buffer_usage.contains(BufferUsage::STORAGE) {
                STORAGE_BUFFER_SIZE
            } else {
                info.size
            };
            let gl = &self.device.get_context();
            let id = gl_call!(gl.create_buffer())
                .ok_or("failed to create_buffer")
                .unwrap();
            gl_call!(gl.bind_buffer(Gl::UNIFORM_BUFFER, Some(&id)));
            let type_ = if info
                .buffer_usage
                .contains(BufferUsage::COPY_DST | BufferUsage::INDIRECT)
            {
                Gl::STREAM_READ
            } else {
                Gl::DYNAMIC_DRAW
            };
            gl_call!(gl.buffer_data_with_i32(Gl::UNIFORM_BUFFER, size as i32, type_));
            Buffer::WebGlBuffer(id)
        };
        let gl_buffer_info = GlBufferInfo { buffer, info };
        self.resources
            .buffers
            .write()
            .insert(buffer_id, gl_buffer_info);
        buffer_id
    }

    fn write_mapped_buffer(
        &self,
        id: BufferId,
        range: Range<u64>,
        write: &mut dyn FnMut(&mut [u8], &dyn RenderResourceContext),
    ) {
        // TODO - for in-memory buffers find a way how to write
        // directly to buffer (it is problematic, as write callback may call
        // may call create_buffer, which locks resources.buffers)
        let size = (range.end - range.start) as usize;
        let mut data = Vec::with_capacity(size);
        unsafe { data.set_len(size) }
        write(&mut data, self);

        let mut buffers = self.resources.buffers.write();
        let buffer = buffers.get_mut(&id).unwrap();

        match &mut buffer.buffer {
            Buffer::WebGlBuffer(buffer_id) => {
                let gl = &self.device.get_context();
                gl_call!(gl.bind_buffer(Gl::COPY_WRITE_BUFFER, Some(&buffer_id)));
                gl_call!(
                    gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                        Gl::COPY_WRITE_BUFFER,
                        range.start as i32,
                        &data,
                        0,
                        data.len() as u32,
                    )
                );
            }
            Buffer::Data(buffer_data) => {
                let sub_data =
                    &mut buffer_data.as_mut_slice()[(range.start as usize)..(range.end as usize)];
                sub_data.copy_from_slice(&data);
            }
        }
        // info!(
        //     "done write_mapped_buffer, short_id: {:?}",
        //     self.resources.short_buffer_id(id)
        // );
    }

    fn read_mapped_buffer(
        &self,
        id: BufferId,
        range: Range<u64>,
        read: &dyn Fn(&[u8], &dyn RenderResourceContext),
    ) {
        // let gl = &self.device.get_context();
        // let fence_sync = self.resources.fence_sync.read();
        // if fence_sync.is_some() {
        //     let sync = fence_sync.as_ref().unwrap();
        //     // let ret = gl.client_wait_sync_with_u32(&sync, Gl::SYNC_FLUSH_COMMANDS_BIT, 0);
        //     let ret = gl.client_wait_sync_with_u32(sync, Gl::SYNC_FLUSH_COMMANDS_BIT, 0);
        //     match ret {
        //         Gl::ALREADY_SIGNALED => info!("already signaled"),
        //         Gl::CONDITION_SATISFIED => info!("condition satisfied"),
        //         Gl::TIMEOUT_EXPIRED => info!("timeout expired"),
        //         _ => info!("unknown err"),
        //     }
        // } else {
        //     info!("no fence sync");
        // }

        let mut buffers = self.resources.buffers.write();
        let info = buffers.get_mut(&id).unwrap();
        if let Buffer::WebGlBuffer(buffer_id) = &info.buffer {
            let mut data: Vec<u8> = Vec::with_capacity((range.end - range.start) as usize);
            unsafe {
                data.set_len((range.end - range.start) as usize);
            }
            let gl = &self.device.get_context();
            gl.bind_buffer(Gl::PIXEL_PACK_BUFFER, Some(buffer_id));
            gl.get_buffer_sub_data_with_i32_and_u8_array(Gl::PIXEL_PACK_BUFFER, 0, data.as_mut());
            read(data.as_mut(), self);
        }
    }

    fn map_buffer(&self, _id: BufferId, _mode: BufferMapMode) {
        // info!("map buffer {:?}", _id);
    }

    fn unmap_buffer(&self, _id: BufferId) {
        // info!("unmap buffer {:?}", _id);
    }

    fn create_buffer_with_data(&self, info: BufferInfo, data: &[u8]) -> BufferId {
        let buffer_id = BufferId::new();
        trace!(
            "create_buffer_with_data, info: {:?}, short_id: {:?}, buffer_id: {:?}",
            info,
            self.resources.short_buffer_id(buffer_id),
            buffer_id,
        );
        let mut info = info;
        if info.size == 0 {
            info.size = data.len();
        }
        let buffer = if info
            .buffer_usage
            .contains(BufferUsage::MAP_WRITE | BufferUsage::COPY_SRC)
        {
            // in-memory buffer
            Buffer::Data(Vec::from(data))
        } else {
            let gl = &self.device.get_context();
            let id = gl_call!(gl.create_buffer())
                .ok_or("failed to create_buffer")
                .unwrap();
            if info.buffer_usage & BufferUsage::VERTEX == BufferUsage::VERTEX {
                gl_call!(gl.bind_buffer(Gl::ARRAY_BUFFER, Some(&id)));
                gl_call!(gl.buffer_data_with_u8_array(Gl::ARRAY_BUFFER, &data, Gl::DYNAMIC_DRAW));
            } else if info.buffer_usage & BufferUsage::INDEX == BufferUsage::INDEX {
                gl_call!(gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(&id)));
                gl_call!(gl.buffer_data_with_u8_array(
                    Gl::ELEMENT_ARRAY_BUFFER,
                    &data,
                    Gl::DYNAMIC_DRAW
                ));
            } else {
                gl_call!(gl.bind_buffer(Gl::PIXEL_UNPACK_BUFFER, Some(&id)));
                gl_call!(gl.buffer_data_with_u8_array(
                    Gl::PIXEL_UNPACK_BUFFER,
                    &data,
                    Gl::DYNAMIC_DRAW
                ));
            };
            Buffer::WebGlBuffer(id)
        };

        let gl_buffer_info = GlBufferInfo { buffer, info };
        self.resources
            .buffers
            .write()
            .insert(buffer_id, gl_buffer_info);
        buffer_id
    }

    fn create_shader_module(&self, _shader_handle: &Handle<Shader>, _shaders: &Assets<Shader>) {}

    fn remove_buffer(&self, buffer: BufferId) {
        let gl = &self.device.get_context();
        let mut buffers = self.resources.buffers.write();
        let gl_buffer = buffers.remove(&buffer).unwrap();
        if let Buffer::WebGlBuffer(buffer_id) = &gl_buffer.buffer {
            gl_call!(gl.delete_buffer(Some(buffer_id)));
        }
    }

    fn remove_texture(&self, texture: TextureId) {
        let gl = &self.device.get_context();
        let mut texture_descriptors = self.resources.texture_descriptors.write();
        let mut textures = self.resources.textures.write();
        let gl_texture = textures.remove(&texture).unwrap();
        gl_call!(gl.delete_texture(Some(&gl_texture)));
        texture_descriptors.remove(&texture);
    }

    fn remove_sampler(&self, _sampler: SamplerId) {}

    fn set_asset_resource_untyped(
        &self,
        handle: HandleUntyped,
        render_resource: RenderResourceId,
        index: u64,
    ) {
        let mut asset_resources = self.resources.asset_resources.write();
        asset_resources.insert((handle, index), render_resource);
    }

    fn get_asset_resource_untyped(
        &self,
        handle: HandleUntyped,
        index: u64,
    ) -> Option<RenderResourceId> {
        let asset_resources = self.resources.asset_resources.read();
        asset_resources.get(&(handle, index)).cloned()
    }

    fn remove_asset_resource_untyped(&self, handle: HandleUntyped, index: u64) {
        let mut asset_resources = self.resources.asset_resources.write();
        asset_resources.remove(&(handle, index));
    }

    fn create_render_pipeline(
        &self,
        pipeline_handle: Handle<PipelineDescriptor>,
        pipeline_descriptor: &PipelineDescriptor,
        _shaders: &Assets<Shader>,
    ) {
        let layout = pipeline_descriptor.get_layout().unwrap();
        for bind_group_descriptor in layout.bind_groups.iter() {
            self.create_bind_group_layout(&bind_group_descriptor);
        }
        let vertex_buffer_descriptors = pipeline_descriptor
            .layout
            .as_ref()
            .unwrap()
            .vertex_buffer_descriptors
            .clone();
        let gl = &self.device.get_context();

        let programs = self.resources.programs.read();
        let program = programs.get(&pipeline_descriptor.shader_stages).unwrap();
        gl_call!(gl.use_program(Some(&program.program)));
        info!("start binding");
        for bind_group in layout.bind_groups.iter() {
            for binding in bind_group.bindings.iter() {
                let block_index =
                    gl_call!(gl.get_uniform_block_index(&program.program, &binding.name));
                info!("trying to bind {:?}", binding.name);
                if (block_index as i32) < 0 {
                    info!("invalid block index for {:?}, skipping", &binding.name);
                    if let Some(uniform_location) =
                        gl_call!(gl.get_uniform_location(&program.program, &binding.name))
                    {
                        info!("found uniform location: {:?}", uniform_location);
                        if let BindType::Texture { .. } = binding.bind_type {
                            let texture_unit = self
                                .resources
                                .get_or_create_texture_unit(bind_group.index, binding.index);
                            gl_call!(gl.uniform1i(Some(&uniform_location), texture_unit as i32));
                            info!(
                                "found texture uniform {:?}, binding to unit {:?}",
                                binding.name, texture_unit
                            );
                        } else {
                            panic!("use non-block uniforms expected only for textures");
                        }
                    } else {
                        info!("can't bind {:?}", binding.name);
                    }
                    continue;
                }
                let binding_point = self
                    .resources
                    .get_or_create_binding_point(bind_group.index, binding.index);
                gl_call!(gl.uniform_block_binding(&program.program, block_index, binding_point));
                let _min_data_size = gl_call!(gl.get_active_uniform_block_parameter(
                    &program.program,
                    block_index,
                    Gl::UNIFORM_BLOCK_DATA_SIZE,
                ))
                .unwrap();
                info!(
                    "uniform_block_binding: name: {:?}, block_index: {:?}, binding_point: {:?}, min_data_size: {:?}",
                    binding.name,
                    block_index,
                    binding_point,
                    _min_data_size,
                );
            }
        }
        info!("done binding");
        info!("vertex_buffer_descriptors: {:?}", vertex_buffer_descriptors);
        let vertex_buffer_descriptors = vertex_buffer_descriptors
            .iter()
            .map(|vertex_buffer_descriptor| {
                GlVertexBufferDescripror::from(gl, &program.program, vertex_buffer_descriptor)
            })
            .collect();
        let vao = gl_call!(gl.create_vertex_array()).unwrap();
        let pipeline = WebGL2Pipeline {
            shader_stages: pipeline_descriptor.shader_stages.clone(),
            vertex_buffer_descriptors,
            vao,
            update_vao: false,
            index_buffer: None,
            index_format: IndexFormat::Uint32,
            vertex_buffer: None,
            color_target_states: pipeline_descriptor.color_target_states.clone(),
            depth_stencil: pipeline_descriptor.depth_stencil.clone(),
            primitive: pipeline_descriptor.primitive.clone(),
            scissors_state: None,
        };
        self.pipeline_descriptors
            .write()
            .insert(pipeline_handle.clone(), pipeline_descriptor.clone());
        self.resources
            .pipelines
            .write()
            .insert(pipeline_handle, pipeline);
    }

    fn create_bind_group(
        &self,
        bind_group_descriptor_id: BindGroupDescriptorId,
        bind_group: &BindGroup,
    ) {
        assert!(self.bind_group_descriptor_exists(bind_group_descriptor_id));
        let layouts = self.resources.bind_group_layouts.read();
        let bind_group_layout = layouts.get(&bind_group_descriptor_id).unwrap();
        let _gl = &self.device.get_context();
        let mut bind_groups = self.resources.bind_groups.write();
        if bind_groups.get(&bind_group.id).is_some() {
            return;
        }
        let bind_group_vec: Vec<_> = bind_group
            .indexed_bindings
            .iter()
            .filter(|entry| {
                entry.entry.get_buffer().is_some() || entry.entry.get_texture().is_some()
            }) // TODO
            .map(|entry| match &entry.entry {
                RenderResourceBinding::Buffer { buffer, range, .. } => {
                    let binding_point = self
                        .resources
                        .get_or_create_binding_point(bind_group_layout.index, entry.index);
                    WebGL2RenderResourceBinding::Buffer {
                        binding_point,
                        buffer: *buffer,
                        range: range.clone(),
                    }
                }
                RenderResourceBinding::Texture(texture) => {
                    let texture_unit = self
                        .resources
                        .get_or_create_texture_unit(bind_group_layout.index, entry.index);
                    WebGL2RenderResourceBinding::Texture {
                        texture: *texture,
                        texture_unit,
                    }
                }
                RenderResourceBinding::Sampler(sampler) => {
                    WebGL2RenderResourceBinding::Sampler(*sampler)
                }
            })
            .collect();
        bind_groups.insert(bind_group.id, bind_group_vec);
    }

    fn create_shader_module_from_source(&self, _shader_handle: &Handle<Shader>, _shader: &Shader) {}

    fn clear_bind_groups(&self) {
        self.resources.bind_groups.write().clear();
    }

    fn get_buffer_info(&self, buffer: BufferId) -> Option<BufferInfo> {
        self.resources
            .buffers
            .read()
            .get(&buffer)
            .map(|f| f.info.clone())
    }

    fn bind_group_descriptor_exists(
        &self,
        bind_group_descriptor_id: BindGroupDescriptorId,
    ) -> bool {
        return self
            .resources
            .bind_group_layouts
            .read()
            .contains_key(&bind_group_descriptor_id);
    }
    fn get_specialized_shader(
        &self,
        shader: &Shader,
        macros: Option<&[String]>,
    ) -> Result<Shader, ShaderError> {
        if let ShaderSource::Glsl(source) = &shader.source {
            let source = source.trim_start();
            assert!(source.starts_with("#version"));
            let eol_index = source.find('\n').unwrap();
            let (version_str, source) = source.split_at(eol_index);
            let mut processed = version_str.to_string();
            processed.push_str("\n");
            if let Some(macros) = macros {
                for m in macros.iter() {
                    processed.push_str(&format!("#define {}\n", m));
                }
            }
            processed.push_str("#define WEBGL\n");
            processed.push_str(source);
            Ok(Shader {
                source: ShaderSource::Glsl(processed),
                ..*shader
            })
        } else {
            panic!("spirv shader is not supported");
        }
    }
    fn remove_stale_bind_groups(&self) {}
}
