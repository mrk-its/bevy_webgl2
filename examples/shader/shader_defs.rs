use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::shape,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{asset_shader_defs_system, ShaderDefs, ShaderStage, ShaderStages},
    },
};

/// This example illustrates how to create a custom material asset that uses "shader defs" and a shader that uses that material.
/// In Bevy, "shader defs" are a way to selectively enable parts of a shader based on values set in a component or asset.
fn main() {
    App::build()
        .add_plugins(bevy_webgl2::DefaultPlugins)
        .add_asset::<MyMaterial>()
        .add_startup_system(setup.system())
        .add_system_to_stage(
            CoreStage::PostUpdate,
            asset_shader_defs_system::<MyMaterial>.system(),
        )
        .run();
}

#[derive(RenderResources, ShaderDefs, Default, TypeUuid)]
#[uuid = "620f651b-adbe-464b-b740-ba0e547282ba"]
struct MyMaterial {
    pub color: Color,
    #[render_resources(ignore)]
    #[shader_def]
    pub always_blue: bool,
}

const VERTEX_SHADER: &str = r#"
#version 300 es
precision highp float;
in vec3 Vertex_Position;
layout(std140) uniform CameraViewProj {
    mat4 ViewProj;
};
layout(std140) uniform Transform { // set = 1, binding = 0
    mat4 Model;
};
void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 300 es
precision highp float;
out vec4 o_Target;
layout(std140) uniform MyMaterial_color { // set = 2, binding = 0
    vec4 color;
};

vec4 encodeSRGB(vec4 linearRGB_in) {
    vec3 linearRGB = linearRGB_in.rgb;
    vec3 a = 12.92 * linearRGB;
    vec3 b = 1.055 * pow(linearRGB, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linearRGB);
    return vec4(mix(a, b, c), linearRGB_in.a);
}

void main() {
    o_Target = encodeSRGB(color);

# ifdef MYMATERIAL_ALWAYS_BLUE
    o_Target = encodeSRGB(vec4(0.0, 0.0, 0.8, 1.0));
# endif
}
"#;

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterial>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterial resources to our shader
    render_graph.add_system_node(
        "my_material",
        AssetRenderResourcesNode::<MyMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge("my_material", base::node::MAIN_PASS)
        .unwrap();

    // Create a green material
    let green_material = materials.add(MyMaterial {
        color: Color::rgb(0.0, 0.8, 0.0),
        always_blue: false,
    });

    // Create a blue material, which uses our "always_blue" shader def
    let blue_material = materials.add(MyMaterial {
        color: Color::rgb(0.0, 0.0, 0.0),
        always_blue: true,
    });

    // Create a cube mesh which will use our materials
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 2.0 }));

    commands
        // cube
        .spawn_bundle(MeshBundle {
            mesh: cube_handle.clone(),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle.clone(),
            )]),
            transform: Transform::from_translation(Vec3::new(-2.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(green_material);
    // cube
    commands
        .spawn_bundle(MeshBundle {
            mesh: cube_handle,
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(blue_material);
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(3.0, 5.0, -8.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}
