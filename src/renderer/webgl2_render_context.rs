use super::{Gl, WebGL2RenderResourceContext};
use crate::converters::*;
use crate::{gl_call, Buffer, WebGL2RenderPass};
use bevy::render::{
    pass::{LoadOp, PassDescriptor, RenderPass, TextureAttachment},
    renderer::{BufferId, RenderContext, RenderResourceBindings, RenderResourceContext, TextureId},
    texture::Extent3d,
};
use js_sys::Object;
use std::sync::Arc;
use wasm_bindgen::JsValue;

pub struct WebGL2RenderContext {
    pub device: Arc<crate::Device>,
    pub render_resource_context: WebGL2RenderResourceContext,
}

impl WebGL2RenderContext {
    pub fn new(device: Arc<crate::Device>, resources: WebGL2RenderResourceContext) -> Self {
        WebGL2RenderContext {
            device,
            render_resource_context: resources,
        }
    }

    /// Consume this context, finalize the current CommandEncoder (if it exists), and take the current WebGL2Resources.
    /// This is intended to be called from a worker thread right before synchronizing with the main thread.
    pub fn finish(&mut self) {}
}

impl RenderContext for WebGL2RenderContext {
    fn copy_buffer_to_buffer(
        &mut self,
        source_buffer: BufferId,
        source_offset: u64,
        destination_buffer: BufferId,
        destination_offset: u64,
        size: u64,
    ) {
        let gl = &self.device.get_context();
        let resources = &self.render_resource_context.resources;
        let buffers = resources.buffers.read();
        let src = buffers.get(&source_buffer).unwrap();
        let dst = buffers.get(&destination_buffer).unwrap();
        match (&src.buffer, &dst.buffer) {
            (Buffer::WebGlBuffer(src_id), Buffer::WebGlBuffer(dst_id)) => {
                gl_call!(gl.bind_buffer(Gl::COPY_READ_BUFFER, Some(&src_id)));
                gl_call!(gl.bind_buffer(Gl::COPY_WRITE_BUFFER, Some(&dst_id)));
                gl_call!(gl.copy_buffer_sub_data_with_i32_and_i32_and_i32(
                    Gl::COPY_READ_BUFFER,
                    Gl::COPY_WRITE_BUFFER,
                    source_offset as i32,
                    destination_offset as i32,
                    size as i32,
                ));
            }
            (Buffer::Data(data), Buffer::WebGlBuffer(dst_id)) => {
                gl_call!(gl.bind_buffer(Gl::COPY_WRITE_BUFFER, Some(dst_id)));
                (gl.buffer_sub_data_with_i32_and_u8_array_and_src_offset_and_length(
                    Gl::COPY_WRITE_BUFFER,
                    destination_offset as i32,
                    data,
                    source_offset as u32,
                    size as u32,
                ));
            }
            _ => panic!("copy_buffer_to_buffer: writing to in-memory buffer is not supported"),
        }
    }

    fn copy_buffer_to_texture(
        &mut self,
        source_buffer: BufferId,
        source_offset: u64,
        _source_bytes_per_row: u32,
        destination_texture: TextureId,
        _destination_origin: [u32; 3],
        _destination_mip_level: u32,
        size: Extent3d,
    ) {
        let gl = &self.device.get_context();
        let resources = &self.render_resource_context.resources;
        let textures = resources.textures.read();
        let texture = textures.get(&destination_texture).unwrap();
        let buffers = resources.buffers.read();
        let buffer = buffers.get(&source_buffer).unwrap();
        let texture_descriptors = resources.texture_descriptors.read();
        let texture_descriptor = texture_descriptors.get(&destination_texture).unwrap();
        // TODO
        // let tex_internal_format = match &texture_descriptor.format {
        //     TextureFormat::Rgba8UnormSrgb => Gl::RGBA8_SNORM,
        //     TextureFormat::Rgba8Snorm => Gl::RGBA8_SNORM,
        //     _ => Gl::RGBA,
        // };

        gl_call!(gl.bind_texture(Gl::TEXTURE_2D, Some(&texture)));
        let (internal_format, format, _type) = texture_descriptor.format.webgl2_into();

        match &buffer.buffer {
            Buffer::WebGlBuffer(buffer_id) => {
                gl_call!(gl.bind_buffer(Gl::PIXEL_UNPACK_BUFFER, Some(buffer_id)));
                gl_call!(
                    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_f64(
                        Gl::TEXTURE_2D,
                        0,                      //destination_mip_level as i32,
                        internal_format as i32, // TODO
                        size.width as i32,
                        size.height as i32,
                        0,
                        format,
                        _type,
                        source_offset as f64,
                    )
                )
                .expect("tex image");
            }
            Buffer::Data(data) => {
                let buffer: Object = unsafe {
                    let len = data.len() / 4;
                    let buffer = &std::mem::transmute::<&[u8], &[u32]>(&data)[..len];
                    js_sys::Uint32Array::view(buffer).into()
                };
                gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset(
                    Gl::TEXTURE_2D,
                    0,                      //destination_mip_level as i32,
                    internal_format as i32, // TODO
                    size.width as i32,
                    size.height as i32,
                    0,
                    format,
                    _type,
                    &buffer,
                    0,
                ).expect("tex image");
            }
        };
        // gl_call!(gl.generate_mipmap(Gl::TEXTURE_2D));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MIN_FILTER, Gl::NEAREST as i32));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_MAG_FILTER, Gl::NEAREST as i32));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_S, Gl::CLAMP_TO_EDGE as i32));
        gl_call!(gl.tex_parameteri(Gl::TEXTURE_2D, Gl::TEXTURE_WRAP_T, Gl::CLAMP_TO_EDGE as i32));
    }

    fn copy_texture_to_buffer(
        &mut self,
        source_texture: TextureId,
        source_origin: [u32; 3],
        _source_mip_level: u32,
        destination_buffer: BufferId,
        _destination_offset: u64,
        _destination_bytes_per_row: u32,
        size: Extent3d,
    ) {
        let gl = &self.device.get_context();
        let resources = &mut self.render_resource_context.resources;
        let buffers = resources.buffers.read();
        let dst = buffers.get(&destination_buffer).unwrap();
        let texture_descriptors = resources.texture_descriptors.read();
        let texture_descriptor = texture_descriptors.get(&source_texture).unwrap();

        // TODO add recommended read format
        let (_, _, _type) = texture_descriptor.format.webgl2_into();
        let read_fmt = match _type {
            Gl::UNSIGNED_INT => Gl::RGBA_INTEGER,
            Gl::INT => Gl::RGBA_INTEGER,
            Gl::UNSIGNED_BYTE => Gl::RGBA,
            _ => panic!("not supported read_pixels fmt"),
        };

        let mut framebuffers = resources.framebuffers.write();
        if let Some(fb) = framebuffers.get(&source_texture) {
            gl_call!(gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&fb)));
        } else {
            let textures = resources.textures.read();
            let gl_texture = textures.get(&source_texture);
            let fb = gl_call!(gl.create_framebuffer()).unwrap();
            gl_call!(gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&fb)));
            framebuffers.insert(source_texture, fb);
            gl_call!(gl.framebuffer_texture_2d(
                Gl::FRAMEBUFFER,
                Gl::COLOR_ATTACHMENT0 as u32,
                Gl::TEXTURE_2D,
                gl_texture,
                0,
            ));
            assert!(gl.check_framebuffer_status(Gl::FRAMEBUFFER) == Gl::FRAMEBUFFER_COMPLETE);
        }
        if let Buffer::WebGlBuffer(dst_id) = &dst.buffer {
            gl_call!(gl.bind_buffer(Gl::PIXEL_PACK_BUFFER, Some(&dst_id)));
            gl_call!(gl.read_buffer(Gl::COLOR_ATTACHMENT0));
            // bevy::utils::tracing::info!("read_pixels, width: {}, height: {}, fmt: {}, type: {}", size.width, size.height, format, _type);
            gl_call!(gl.read_pixels_with_i32(
                source_origin[0] as i32,
                source_origin[1] as i32,
                size.width as i32,
                size.height as i32,
                read_fmt,
                _type,
                0,
            ))
            .unwrap();
            // let sync = gl.fence_sync(Gl::SYNC_GPU_COMMANDS_COMPLETE, 0).unwrap();
            // gl.client_wait_sync_with_u32(&sync, Gl::SYNC_FLUSH_COMMANDS_BIT, 0);
            // let mut fence_sync = resources.fence_sync.write();
            // if fence_sync.is_some() {
            //     gl.delete_sync(Some(fence_sync.as_ref().unwrap()))
            // }
            // fence_sync.replace(sync);
            // bevy::utils::tracing::info!("created fence sync");
        }
    }

    fn copy_texture_to_texture(
        &mut self,
        _source_texture: TextureId,
        _source_origin: [u32; 3],
        _source_mip_level: u32,
        _destination_texture: TextureId,
        _destination_origin: [u32; 3],
        _destination_mip_level: u32,
        _size: Extent3d,
    ) {
        unimplemented!()
    }

    fn resources(&self) -> &dyn RenderResourceContext {
        &self.render_resource_context
    }

    fn resources_mut(&mut self) -> &mut dyn RenderResourceContext {
        &mut self.render_resource_context
    }

    fn begin_pass(
        &mut self,
        pass_descriptor: &PassDescriptor,
        _render_resource_bindings: &RenderResourceBindings,
        run_pass: &mut dyn FnMut(&mut dyn RenderPass),
    ) {
        let gl = &self.device.get_context();
        gl_call!(gl.disable(Gl::SCISSOR_TEST));
        let texture_id = if let TextureAttachment::Id(texture_id) =
            &pass_descriptor.color_attachments[0].attachment
        {
            texture_id
        } else {
            panic!("first attachment must be a texture");
        };
        let is_swapchain = texture_id == &self.render_resource_context.swapchain_texture
            || pass_descriptor.color_attachments[0]
                .resolve_target
                .is_some();
        // info!("pass_descriptor: {:#?}", pass_descriptor);
        if is_swapchain {
            gl_call!(gl.bind_framebuffer(
                Gl::FRAMEBUFFER,
                None as Option<&crate::renderer::WebGlFramebuffer>
            ));
            let window_size = self.render_resource_context.resources.window_size.read();
            gl_call!(gl.viewport(0, 0, window_size.0 as i32, window_size.1 as i32));
            let mut mask = 0;
            if let LoadOp::Clear(c) = &pass_descriptor.color_attachments[0].ops.load {
                gl_call!(gl.clear_color(c.r(), c.g(), c.b(), c.a()));
                mask |= Gl::COLOR_BUFFER_BIT;
            }
            if let Some(d) = &pass_descriptor.depth_stencil_attachment {
                if let Some(x) = &d.depth_ops {
                    if let LoadOp::Clear(value) = x.load {
                        // it seems it has no effect
                        gl_call!(gl.clear_depth(value));
                        mask |= Gl::DEPTH_BUFFER_BIT;
                    }
                }
            }
            if mask > 0 {
                gl_call!(gl.clear(mask));
            }
        } else {
            let textures = self.render_resource_context.resources.textures.read();
            let texture_info = self
                .render_resource_context
                .resources
                .texture_descriptors
                .read();
            let mut framebuffers = self.render_resource_context.resources.framebuffers.write();
            if let Some(fb) = framebuffers.get(texture_id) {
                gl_call!(gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&fb)));
            } else {
                let fb = gl_call!(gl.create_framebuffer()).unwrap();
                gl_call!(gl.bind_framebuffer(Gl::FRAMEBUFFER, Some(&fb)));
                framebuffers.insert(*texture_id, fb);
                let draw_buffers = JsValue::from(
                    pass_descriptor
                        .color_attachments
                        .iter()
                        .enumerate()
                        .map(|(n, descr)| match descr.attachment {
                            TextureAttachment::Id(_) => Gl::COLOR_ATTACHMENT0 + n as u32,
                            _ => Gl::NONE,
                        })
                        .map(|x| JsValue::from_f64(x as f64))
                        .collect::<js_sys::Array>(),
                );
                gl_call!(gl.draw_buffers(&draw_buffers));
                for (i, descr) in pass_descriptor.color_attachments.iter().enumerate() {
                    if let TextureAttachment::Id(id) = descr.attachment {
                        let gl_texture = textures.get(&id);
                        gl_call!(gl.framebuffer_texture_2d(
                            Gl::FRAMEBUFFER,
                            Gl::COLOR_ATTACHMENT0 + i as u32,
                            Gl::TEXTURE_2D,
                            gl_texture,
                            0,
                        ));
                        assert!(
                            gl.check_framebuffer_status(Gl::FRAMEBUFFER)
                                == Gl::FRAMEBUFFER_COMPLETE
                        );
                    }
                }
            }
            for (i, descr) in pass_descriptor.color_attachments.iter().enumerate() {
                if i == 0 {
                    if let TextureAttachment::Id(id) = descr.attachment {
                        let texture_descr = texture_info.get(&id).unwrap();
                        gl_call!(gl.viewport(
                            0,
                            0,
                            texture_descr.size.width as i32,
                            texture_descr.size.height as i32,
                        ));
                    }
                }
                if let LoadOp::Clear(c) = descr.ops.load {
                    // TODO - use proper function / color scalling depending
                    // on texture format
                    if i == 0 {
                        gl_call!(gl.clear_bufferfv_with_f32_array(
                            Gl::COLOR,
                            i as i32,
                            &[c.r(), c.g(), c.b(), c.a()]
                        ));
                    } else {
                        gl_call!(gl.clear_bufferuiv_with_u32_array_and_src_offset(
                            Gl::COLOR,
                            i as i32,
                            &[c.r() as u32, c.g() as u32, c.b() as u32, c.a() as u32],
                            0
                        ));
                    }
                }
            }
        }

        let mut render_pass = WebGL2RenderPass {
            render_context: self,
            pipeline: None,
        };
        run_pass(&mut render_pass);
    }
}
