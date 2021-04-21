use super::{Gl, WebGl2RenderingContext};
use crate::{gl_call, GlBindGroups, GlProgram, GlShader, GlVertexFormat};
use bevy::log::prelude::*;
use bevy::render::{
    pipeline::{
        BindGroupDescriptor, BindType, BindingDescriptor, BindingShaderStage, InputStepMode,
        PipelineLayout, UniformProperty, VertexAttribute, VertexBufferLayout, VertexFormat,
    },
    texture::{TextureSampleType, TextureViewDimension},
};
use bevy::utils::HashSet;
use std::iter::Extend;
use web_sys::WebGlActiveInfo;

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<GlShader, String> {
    let mut bind_groups = GlBindGroups::default();
    let bind_group_re =
        regex::Regex::new(r"uniform\s+(\w+)\s*\{\s*//\s*set\s*=\s*(\d+)[, ]+binding\s*=\s*(\d+)")
            .unwrap();
    for cap in bind_group_re.captures_iter(source) {
        bind_groups.insert(
            cap[1].to_string(),
            (cap[2].parse().unwrap(), cap[3].parse().unwrap()),
        );
    }

    let bind_group_re =
        regex::Regex::new(r"sampler2D\s+(\w+)\s*;\s*//\s*set\s*=\s*(\d+)[, ]+binding\s*=\s*(\d+)")
            .unwrap();
    for cap in bind_group_re.captures_iter(source) {
        bind_groups.insert(
            cap[1].to_string(),
            (cap[2].parse().unwrap(), cap[3].parse().unwrap()),
        );
    }

    let shader = gl_call!(context.create_shader(shader_type))
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl_call!(context.shader_source(&shader, source));
    gl_call!(context.compile_shader(&shader));

    if gl_call!(context.get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS))
        .as_bool()
        .unwrap_or(false)
    {
        Ok(GlShader::new(shader, bind_groups))
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    shaders: &[GlShader],
) -> Result<GlProgram, String> {
    let program = gl_call!(context.create_program())
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    let mut bind_groups = GlBindGroups::default();
    for shader in shaders {
        gl_call!(context.attach_shader(&program, &shader.shader));
        bind_groups.extend(shader.bind_groups.clone());
    }
    gl_call!(context.link_program(&program));

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(GlProgram::new(program, bind_groups))
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn get_vertex_format(gl_type: u32) -> VertexFormat {
    match gl_type {
        Gl::FLOAT => VertexFormat::Float,
        Gl::FLOAT_VEC2 => VertexFormat::Float2,
        Gl::FLOAT_VEC3 => VertexFormat::Float3,
        Gl::FLOAT_VEC4 => VertexFormat::Float4,
        Gl::INT => VertexFormat::Int,
        Gl::INT_VEC2 => VertexFormat::Int2,
        Gl::INT_VEC3 => VertexFormat::Int3,
        Gl::INT_VEC4 => VertexFormat::Int4,
        Gl::UNSIGNED_INT => VertexFormat::Uint,
        Gl::UNSIGNED_INT_VEC2 => VertexFormat::Uint2,
        Gl::UNSIGNED_INT_VEC3 => VertexFormat::Uint3,
        Gl::UNSIGNED_INT_VEC4 => VertexFormat::Uint4,
        _ => panic!("unknown vertex attribute type: {:?}", gl_type),
    }
}

pub fn reflect_layout(context: &WebGl2RenderingContext, program: &GlProgram) -> PipelineLayout {
    let gl = context;
    let mut shader_location = 0;
    info!("program bind groups: {:?}", program.bind_groups);
    let active_attributes = gl
        .get_program_parameter(&program.program, Gl::ACTIVE_ATTRIBUTES)
        .as_f64()
        .unwrap() as u32;

    let mut vertex_buffer_descriptors = vec![];

    for index in 0..active_attributes {
        let info: WebGlActiveInfo = gl.get_active_attrib(&program.program, index).unwrap();
        let name = info.name();
        if name == "gl_VertexID" || name == "gl_InstanceID" {
            continue;
        }

        let format = get_vertex_format(info.type_());

        vertex_buffer_descriptors.push(VertexBufferLayout {
            name: info.name().into(),
            stride: 0,
            step_mode: InputStepMode::Vertex,
            attributes: vec![VertexAttribute {
                name: info.name().into(),
                offset: 0,
                format,
                shader_location,
            }],
        });
        shader_location += 1;
    }
    let mut bind_groups: Vec<BindGroupDescriptor> = Vec::new();

    let active_uniform_blocks = gl
        .get_program_parameter(&program.program, Gl::ACTIVE_UNIFORM_BLOCKS)
        .as_f64()
        .unwrap() as u32;

    let mut used_indices: HashSet<u32> = HashSet::default();
    used_indices.extend(bind_groups.iter().map(|g| g.index));
    used_indices.extend(program.bind_groups.values().map(|(index, _)| *index));

    fn next_group_index(used_indices: &mut HashSet<u32>) -> u32 {
        let mut index = 0;
        while used_indices.contains(&index) {
            index += 1;
        }
        used_indices.insert(index);
        index
    }

    let mut names: Vec<String> = Vec::with_capacity(active_uniform_blocks as usize);

    for uniform_index in 0..active_uniform_blocks {
        let name = gl
            .get_active_uniform_block_name(&program.program, uniform_index)
            .unwrap();

        if name == "CameraPosition" {
            let camera_position = BindingDescriptor {
                name: "CameraPosition".to_string(),
                index: 1,
                bind_type: BindType::Uniform {
                    has_dynamic_offset: false,
                    property: UniformProperty::Struct(vec![UniformProperty::Vec4]),
                },
                shader_stage: BindingShaderStage::FRAGMENT,
            };
            let bind_group = bind_groups.iter_mut().find(|bg| bg.index == 0);
            if let Some(bind_group) = bind_group {
                bind_group.bindings.push(camera_position);
            } else {
                used_indices.insert(0);

                bind_groups.push(BindGroupDescriptor::new(0, vec![camera_position]));
            }
        } else if name == "CameraViewProj" {
            let camera_descriptor = BindingDescriptor {
                name: "CameraViewProj".to_string(),
                index: 0,
                bind_type: BindType::Uniform {
                    has_dynamic_offset: false,
                    property: UniformProperty::Struct(vec![UniformProperty::Mat4]),
                },
                shader_stage: BindingShaderStage::VERTEX | BindingShaderStage::FRAGMENT,
            };
            let bind_group = bind_groups.iter_mut().find(|bg| bg.index == 0);
            if let Some(bind_group) = bind_group {
                bind_group.bindings.push(camera_descriptor);
            } else {
                used_indices.insert(0);

                bind_groups.push(BindGroupDescriptor::new(0, vec![camera_descriptor]));
            }
        }

        names.push(name);
    }

    for uniform_index in 0..active_uniform_blocks {
        let name = &names[uniform_index as usize];

        if name == "CameraPosition" || name == "CameraViewProj" {
            continue;
        }

        let size = gl
            .get_active_uniform_block_parameter(
                &program.program,
                uniform_index,
                Gl::UNIFORM_BLOCK_DATA_SIZE,
            )
            .unwrap()
            .as_f64()
            .unwrap() as u32;
        // let active_uniforms = gl
        //     .get_active_uniform_block_parameter(
        //         &program,
        //         uniform_index,
        //         Gl::UNIFORM_BLOCK_ACTIVE_UNIFORMS,
        //     )
        //     .unwrap()
        //     .as_f64()
        //     .unwrap() as u32;
        // let active_uniform_indices = gl
        //     .get_active_uniform_block_parameter(
        //         &program,
        //         uniform_index,
        //         Gl::UNIFORM_BLOCK_ACTIVE_UNIFORM_INDICES,
        //     )
        //     .unwrap();

        // trace!(
        //     "index: {:?}, name: {:?} size: {:?} active_uniforms: {:?} indices: {:?}",
        //     uniform_index,
        //     name,
        //     size,
        //     active_uniforms,
        //     active_uniform_indices
        // );
        let (group_index, index) = if let Some((group_index, index)) = program.bind_groups.get(name)
        {
            (*group_index, *index)
        } else {
            (next_group_index(&mut used_indices), 0)
        };
        let property = UniformProperty::Array(Box::new(UniformProperty::UInt), size as usize / 4);
        let binding = BindingDescriptor {
            name: name.to_string(),
            index,
            bind_type: BindType::Uniform {
                has_dynamic_offset: false,
                property,
            },
            shader_stage: BindingShaderStage::VERTEX | BindingShaderStage::FRAGMENT,
        };

        let bind_group = bind_groups.iter_mut().find(|bg| bg.index == group_index);
        if let Some(bind_group) = bind_group {
            bind_group.bindings.push(binding);
        } else {
            bind_groups.push(BindGroupDescriptor::new(group_index, vec![binding]));
        }
    }

    let active_uniforms = gl
        .get_program_parameter(&program.program, Gl::ACTIVE_UNIFORMS)
        .as_f64()
        .unwrap() as u32;
    for uniform_index in 0..active_uniforms {
        let info = gl
            .get_active_uniform(&program.program, uniform_index)
            .unwrap();
        let name = info.name();

        if [
            Gl::SAMPLER_2D,
            Gl::UNSIGNED_INT_SAMPLER_2D,
            Gl::INT_SAMPLER_2D,
        ]
        .contains(&info.type_())
        {
            let (group_index, index) =
                if let Some((group_index, index)) = program.bind_groups.get(&name) {
                    (*group_index, *index)
                } else {
                    (next_group_index(&mut used_indices), 0)
                };

            let binding = BindingDescriptor {
                name: info.name(),
                index: index,
                bind_type: BindType::Texture {
                    multisampled: false,
                    view_dimension: TextureViewDimension::D2,
                    sample_type: TextureSampleType::Float { filterable: true },
                },
                shader_stage: BindingShaderStage::FRAGMENT,
            };
            let bind_group = bind_groups.iter_mut().find(|bg| bg.index == group_index);
            if let Some(bind_group) = bind_group {
                bind_group.bindings.push(binding);
            } else {
                bind_groups.push(BindGroupDescriptor::new(group_index, vec![binding]));
            }
        }
    }
    bind_groups.sort_by_key(|g| g.index);
    for bind_group in bind_groups.iter_mut() {
        bind_group.bindings.sort_by_key(|b| b.index);
    }
    PipelineLayout {
        bind_groups,
        vertex_buffer_descriptors,
    }
}

pub fn gl_vertex_format(vertex_format: &VertexFormat) -> GlVertexFormat {
    let (format, nr_of_components, normalized) = match vertex_format {
        VertexFormat::Uchar2 => (Gl::BYTE, 2, false),
        VertexFormat::Uchar4 => (Gl::BYTE, 4, false),
        VertexFormat::Char2 => (Gl::BYTE, 2, false),
        VertexFormat::Char4 => (Gl::BYTE, 4, false),
        VertexFormat::Uchar2Norm => (Gl::BYTE, 2, true),
        VertexFormat::Uchar4Norm => (Gl::BYTE, 4, true),
        VertexFormat::Char2Norm => (Gl::BYTE, 2, true),
        VertexFormat::Char4Norm => (Gl::BYTE, 4, true),
        VertexFormat::Ushort2 => (Gl::UNSIGNED_SHORT, 2, false),
        VertexFormat::Ushort4 => (Gl::UNSIGNED_SHORT, 4, false),
        VertexFormat::Short2 => (Gl::SHORT, 2, false),
        VertexFormat::Short4 => (Gl::SHORT, 4, false),
        VertexFormat::Ushort2Norm => (Gl::UNSIGNED_SHORT, 2, true),
        VertexFormat::Ushort4Norm => (Gl::UNSIGNED_SHORT, 4, true),
        VertexFormat::Short2Norm => (Gl::SHORT, 2, true),
        VertexFormat::Short4Norm => (Gl::SHORT, 4, true),
        VertexFormat::Half2 => (Gl::HALF_FLOAT, 2, false),
        VertexFormat::Half4 => (Gl::HALF_FLOAT, 4, false),
        VertexFormat::Float => (Gl::FLOAT, 1, false),
        VertexFormat::Float2 => (Gl::FLOAT, 2, false),
        VertexFormat::Float3 => (Gl::FLOAT, 3, false),
        VertexFormat::Float4 => (Gl::FLOAT, 4, false),
        VertexFormat::Uint => (Gl::UNSIGNED_INT, 1, false),
        VertexFormat::Uint2 => (Gl::UNSIGNED_INT, 2, false),
        VertexFormat::Uint3 => (Gl::UNSIGNED_INT, 3, false),
        VertexFormat::Uint4 => (Gl::UNSIGNED_INT, 4, false),
        VertexFormat::Int => (Gl::INT, 1, false),
        VertexFormat::Int2 => (Gl::INT, 2, false),
        VertexFormat::Int3 => (Gl::INT, 3, false),
        VertexFormat::Int4 => (Gl::INT, 4, false),
    };
    GlVertexFormat {
        format,
        nr_of_components,
        normalized,
    }
}
