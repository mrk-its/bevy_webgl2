use crate::{gl_call, renderer::*, Buffer, ScissorsState};
use bevy::render::{
    pass::RenderPass,
    pipeline::{
        BindGroupDescriptorId, BlendFactor, BlendOperation, CullMode, FrontFace, IndexFormat,
        PipelineDescriptor,
    },
    renderer::{BindGroupId, BufferId, BufferUsage, RenderContext},
};
use bevy::{asset::Handle, render::pipeline::PrimitiveTopology};
use std::ops::Range;

pub struct WebGL2RenderPass<'a> {
    pub render_context: &'a WebGL2RenderContext,
    pub pipeline: Option<Handle<PipelineDescriptor>>,
}

impl<'a> WebGL2RenderPass<'a> {
    pub fn setup_vao(&self) {
        let gl = &self.render_context.device.get_context();

        let resources = &self.render_context.render_resource_context.resources;
        let mut pipelines = resources.pipelines.write();
        let pipeline_handle = self.pipeline.as_ref().unwrap();
        let pipeline = pipelines.get_mut(&pipeline_handle).unwrap();

        gl_call!(gl.bind_vertex_array(Some(&pipeline.vao)));

        if !pipeline.update_vao {
            return;
        }
        pipeline.update_vao = false;
        let buffers = resources.buffers.write();

        if let Some(buffer_id) = pipeline.vertex_buffer {
            let buffer = buffers.get(&buffer_id).unwrap();
            if let Buffer::WebGlBuffer(buffer_id) = &buffer.buffer {
                gl_call!(gl.bind_buffer(Gl::ARRAY_BUFFER, Some(buffer_id)));
            } else {
                panic!("binding in-memory buffer");
            }
        }

        if let Some(buffer_id) = pipeline.index_buffer {
            let buffer = buffers.get(&buffer_id).unwrap();
            if let Buffer::WebGlBuffer(buffer_id) = &buffer.buffer {
                gl_call!(gl.bind_buffer(Gl::ELEMENT_ARRAY_BUFFER, Some(buffer_id)));
            } else {
                panic!("binding in-memory buffer");
            }
        }

        assert!(pipeline.vertex_buffer_descriptors.len() == 1);
        let vertex_buffer_descriptor = &pipeline.vertex_buffer_descriptors[0];
        for attr_descr in vertex_buffer_descriptor.attributes.iter() {
            if attr_descr.attrib_location >= 0 {
                gl_call!(gl.enable_vertex_attrib_array(attr_descr.attrib_location as u32 as u32));
                if attr_descr.format.format == Gl::FLOAT
                    || attr_descr.format.format == Gl::HALF_FLOAT
                {
                    gl_call!(gl.vertex_attrib_pointer_with_i32(
                        attr_descr.attrib_location as u32,
                        attr_descr.format.nr_of_components,
                        attr_descr.format.format,
                        attr_descr.format.normalized,
                        vertex_buffer_descriptor.stride,
                        attr_descr.offset,
                    ));
                } else {
                    gl_call!(gl.vertex_attrib_i_pointer_with_i32(
                        attr_descr.attrib_location as u32,
                        attr_descr.format.nr_of_components,
                        attr_descr.format.format,
                        vertex_buffer_descriptor.stride,
                        attr_descr.offset,
                    ));
                }
            }
        }
    }
}

impl<'a> RenderPass for WebGL2RenderPass<'a> {
    fn get_render_context(&self) -> &dyn RenderContext {
        self.render_context
    }
    fn set_scissor_rect(&mut self, x: u32, y: u32, w: u32, h: u32) {
        let pipeline_handle = self.pipeline.as_ref().unwrap();
        let mut pipelines = self
            .render_context
            .render_resource_context
            .resources
            .pipelines
            .write();
        let pipeline = pipelines.get_mut(pipeline_handle).unwrap();

        let gl = &self.render_context.device.get_context();

        // TODO - there might be a better way to get a viewport size (make WindowDescriptor available in RenderPass?)
        let viewport: js_sys::Int32Array = gl_call!(gl.get_parameter(Gl::VIEWPORT))
            .unwrap()
            .dyn_into()
            .unwrap();
        let viewport_height = viewport.get_index(3);

        let (x, y, w, h) = (
            x as i32,
            viewport_height - (y + h) as i32,
            w as i32,
            h as i32,
        );
        pipeline.scissors_state = Some(ScissorsState { x, y, w, h });

        gl_call!(gl.enable(Gl::SCISSOR_TEST));
        gl_call!(gl.scissor(x as i32, y as i32, w as i32, h as i32));
    }
    fn set_vertex_buffer(&mut self, _start_slot: u32, buffer_id: BufferId, _offset: u64) {
        // TODO - start_slot and offset parameters
        let resources = &self.render_context.render_resource_context.resources;
        let mut pipelines = resources.pipelines.write();
        let pipeline_handle = self.pipeline.as_ref().unwrap();
        let pipeline = pipelines.get_mut(&pipeline_handle).unwrap();

        if pipeline.vertex_buffer != Some(buffer_id) {
            pipeline.vertex_buffer = Some(buffer_id);
            pipeline.update_vao = true;
        }
    }

    fn set_viewport(&mut self, x: f32, y: f32, w: f32, h: f32, _min_depth: f32, _max_depth: f32) {
        let ctx = &self.render_context;
        let gl = &ctx.device.get_context();
        gl_call!(gl.viewport(x as i32, y as i32, w as i32, h as i32));
    }

    fn set_stencil_reference(&mut self, _reference: u32) {}

    fn set_index_buffer(&mut self, buffer_id: BufferId, _offset: u64, index_format: IndexFormat) {
        // TODO - offset parameter
        // TODO - index_format parameter

        let resources = &self.render_context.render_resource_context.resources;
        let mut pipelines = resources.pipelines.write();
        let pipeline_handle = self.pipeline.as_ref().unwrap();
        let pipeline = pipelines.get_mut(&pipeline_handle).unwrap();

        if pipeline.index_buffer != Some(buffer_id) {
            pipeline.index_buffer = Some(buffer_id);
            pipeline.index_format = index_format;
            pipeline.update_vao = true;
        }
    }

    fn draw_indexed(&mut self, indices: Range<u32>, _base_vertex: i32, instances: Range<u32>) {
        // mysterious "Parking not supported on this platform" panic if you let this code
        // out of its block.
        let (primitives, (index_type, type_size)) = {
            let resources = &self.render_context.render_resource_context.resources;
            let pipelines = resources.pipelines.read();
            let pipeline_handle = self.pipeline.as_ref().unwrap();
            let pipeline = pipelines.get(&pipeline_handle).unwrap();

            let primitives = match pipeline.primitive.topology {
                PrimitiveTopology::PointList => Gl::POINTS,
                PrimitiveTopology::LineList => Gl::LINES,
                PrimitiveTopology::LineStrip => Gl::LINE_STRIP,
                PrimitiveTopology::TriangleList => Gl::TRIANGLES,
                PrimitiveTopology::TriangleStrip => Gl::TRIANGLE_STRIP,
            };

            (
                primitives,
                match pipeline.index_format {
                    IndexFormat::Uint16 => (Gl::UNSIGNED_SHORT, 2),
                    IndexFormat::Uint32 => (Gl::UNSIGNED_INT, 4),
                },
            )
        };

        let ctx = &self.render_context;
        let gl = &ctx.device.get_context();
        self.setup_vao();
        gl_call!(gl.draw_elements_instanced_with_i32(
            primitives,
            (indices.end - indices.start) as i32,
            index_type,
            indices.start as i32 * type_size,
            (instances.end - instances.start) as i32,
        ));
        let gl_null = None;
        gl_call!(gl.bind_vertex_array(gl_null));
    }

    fn draw(&mut self, vertices: Range<u32>, _instances: Range<u32>) {
        let resources = &self.render_context.render_resource_context.resources;
        let pipelines = resources.pipelines.read();
        let pipeline_handle = self.pipeline.as_ref().unwrap();
        let pipeline = pipelines.get(&pipeline_handle).unwrap();
        let ctx = &self.render_context;
        let gl = &ctx.device.get_context();
        self.setup_vao();
        let primitives = match pipeline.primitive.topology {
            PrimitiveTopology::PointList => Gl::POINTS,
            PrimitiveTopology::LineList => Gl::LINES,
            PrimitiveTopology::LineStrip => Gl::LINE_STRIP,
            PrimitiveTopology::TriangleList => Gl::TRIANGLES,
            PrimitiveTopology::TriangleStrip => Gl::TRIANGLE_STRIP,
        };
        gl_call!(gl.draw_arrays(
            primitives,
            vertices.start as i32,
            (vertices.end - vertices.start) as i32
        ));
        let gl_null = None;
        gl_call!(gl.bind_vertex_array(gl_null));
    }

    fn set_bind_group(
        &mut self,
        _index: u32,
        _bind_group_descriptor_id: BindGroupDescriptorId,
        bind_group_id: BindGroupId,
        dynamic_uniform_indices: Option<&[u32]>,
    ) {
        let resources = &self.render_context.render_resource_context.resources;
        let bind_groups = resources.bind_groups.read();
        let bind_group = bind_groups.get(&bind_group_id).unwrap();
        let buffers = resources.buffers.read();
        let textures = resources.textures.read();
        let gl = &self.render_context.device.get_context();
        for (i, binding) in bind_group.iter().enumerate() {
            match binding {
                crate::WebGL2RenderResourceBinding::Buffer {
                    binding_point,
                    buffer,
                    range,
                } => {
                    let offset = *dynamic_uniform_indices
                        .and_then(|indices| indices.get(i))
                        .unwrap_or(&(range.start as u32));
                    let buffer = buffers.get(&buffer).unwrap();
                    let size = if buffer.info.buffer_usage.contains(BufferUsage::STORAGE) {
                        STORAGE_BUFFER_SIZE
                    } else {
                        (range.end - range.start) as usize
                    };
                    if let Buffer::WebGlBuffer(buffer_id) = &buffer.buffer {
                        gl_call!(gl.bind_buffer_range_with_i32_and_i32(
                            Gl::UNIFORM_BUFFER,
                            *binding_point,
                            Some(buffer_id),
                            offset as i32,
                            size as i32,
                        ));
                    } else {
                        panic!("binding in-memory buffer");
                    }
                }
                crate::WebGL2RenderResourceBinding::Texture {
                    texture,
                    texture_unit,
                } => {
                    // it seems it may not work
                    // (forcing texture_unit=1 do not work properly)
                    if let Some(texture) = textures.get(texture) {
                        gl_call!(gl.active_texture(Gl::TEXTURE0 + texture_unit));
                        gl_call!(gl.bind_texture(Gl::TEXTURE_2D, Some(texture)));
                    }
                }
                crate::WebGL2RenderResourceBinding::Sampler(_) => {
                    // TODO
                }
            }
        }
    }

    fn set_pipeline(&mut self, pipeline_handle: &Handle<PipelineDescriptor>) {
        self.pipeline = Some(pipeline_handle.as_weak());

        let resources = &self.render_context.render_resource_context.resources;
        let programs = resources.programs.read();
        let pipelines = resources.pipelines.read();
        let pipeline = pipelines.get(&pipeline_handle).unwrap();
        let ctx = self.render_context;
        let gl = &ctx.device.get_context();

        match &pipeline.primitive.cull_mode {
            CullMode::None => {
                // TODO - can we always keep CULL_FACE enabled
                // and use cullFace(FRONT_AND_BACK)?
                // it seems do not work on contributors example
                gl_call!(gl.disable(Gl::CULL_FACE));
            }
            CullMode::Front => {
                gl_call!(gl.enable(Gl::CULL_FACE));
                gl_call!(gl.cull_face(Gl::FRONT));
            }
            CullMode::Back => {
                gl_call!(gl.enable(Gl::CULL_FACE));
                gl_call!(gl.cull_face(Gl::BACK));
            }
        }

        match &pipeline.primitive.front_face {
            FrontFace::Cw => gl_call!(gl.front_face(Gl::CW)),
            FrontFace::Ccw => gl_call!(gl.front_face(Gl::CCW)),
        }

        if let Some(state) = &pipeline.depth_stencil {
            let depth_func = match state.depth_compare {
                bevy::render::pipeline::CompareFunction::Never => Gl::NEVER,
                bevy::render::pipeline::CompareFunction::Less => Gl::LESS,
                bevy::render::pipeline::CompareFunction::Equal => Gl::EQUAL,
                bevy::render::pipeline::CompareFunction::LessEqual => Gl::LEQUAL,
                bevy::render::pipeline::CompareFunction::Greater => Gl::GREATER,
                bevy::render::pipeline::CompareFunction::NotEqual => Gl::NOTEQUAL,
                bevy::render::pipeline::CompareFunction::GreaterEqual => Gl::GEQUAL,
                bevy::render::pipeline::CompareFunction::Always => Gl::ALWAYS,
            };
            gl_call!(gl.depth_func(depth_func));
        }

        if let Some(state) = pipeline.color_target_states.get(0) {
            gl_call!(gl.enable(Gl::BLEND));
            fn blend_factor(factor: &BlendFactor) -> u32 {
                match factor {
                    BlendFactor::Zero => Gl::ZERO,
                    BlendFactor::One => Gl::ONE,
                    BlendFactor::SrcColor => Gl::SRC_COLOR,
                    BlendFactor::OneMinusSrcColor => Gl::ONE_MINUS_SRC_COLOR,
                    BlendFactor::SrcAlpha => Gl::SRC_ALPHA,
                    BlendFactor::OneMinusSrcAlpha => Gl::ONE_MINUS_SRC_ALPHA,
                    BlendFactor::DstColor => Gl::DST_COLOR,
                    BlendFactor::OneMinusDstColor => Gl::ONE_MINUS_DST_COLOR,
                    BlendFactor::DstAlpha => Gl::DST_ALPHA,
                    BlendFactor::OneMinusDstAlpha => Gl::ONE_MINUS_DST_ALPHA,
                    BlendFactor::SrcAlphaSaturated => Gl::SRC_ALPHA_SATURATE,
                    BlendFactor::BlendColor => Gl::BLEND_COLOR,
                    BlendFactor::OneMinusBlendColor => panic!("not supported"),
                }
            }
            let src_factor = blend_factor(&state.color_blend.src_factor);
            let dst_factor = blend_factor(&state.color_blend.dst_factor);
            let src_alpha = blend_factor(&state.alpha_blend.src_factor);
            let dst_alpha = blend_factor(&state.alpha_blend.dst_factor);
            gl_call!(gl.blend_func_separate(src_factor, dst_factor, src_alpha, dst_alpha));
            let op = match state.color_blend.operation {
                BlendOperation::Add => Gl::FUNC_ADD,
                BlendOperation::Subtract => Gl::FUNC_SUBTRACT,
                BlendOperation::ReverseSubtract => Gl::FUNC_REVERSE_SUBTRACT,
                BlendOperation::Min => Gl::MIN,
                BlendOperation::Max => Gl::MAX,
            };
            gl_call!(gl.blend_equation(op));
        } else {
            gl_call!(gl.disable(Gl::BLEND));
        }

        let program = programs.get(&pipeline.shader_stages).unwrap();

        gl_call!(gl.use_program(Some(&program.program)));

        if let Some(ScissorsState { x, y, w, h }) = pipeline.scissors_state.clone() {
            gl_call!(gl.enable(Gl::SCISSOR_TEST));
            gl_call!(gl.scissor(x, y, w, h));
        } else {
            gl_call!(gl.disable(Gl::SCISSOR_TEST));
        }
    }
}
