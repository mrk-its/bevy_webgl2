use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{shape, VertexAttributeValues},
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};

/// This example illustrates how to add a custom attribute to a mesh and use it in a custom shader.
fn main() {
    App::new()
        .add_plugins(bevy_webgl2::DefaultPlugins)
        .add_asset::<MyMaterialWithVertexColorSupport>()
        .add_startup_system(setup.system())
        .run();
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "0320b9b8-b3a3-4baa-8bfa-c94008177b17"]
struct MyMaterialWithVertexColorSupport {}

const VERTEX_SHADER: &str = r#"
#version 300 es
precision highp float;
in vec3 Vertex_Position;
in vec3 Vertex_Color;
out vec3 v_color;

layout(std140) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(std140) uniform Transform { // set = 1, binding = 0
    mat4 Model;
};

void main() {
    gl_Position = ViewProj * Model * vec4(Vertex_Position, 1.0);
    v_color = Vertex_Color;
}
"#;

const FRAGMENT_SHADER: &str = r#"
#version 300 es
precision highp float;
out vec4 o_Target;
in vec3 v_color;

vec4 encodeSRGB(vec4 linearRGB_in) {
    vec3 linearRGB = linearRGB_in.rgb;
    vec3 a = 12.92 * linearRGB;
    vec3 b = 1.055 * pow(linearRGB, vec3(1.0 / 2.4)) - 0.055;
    vec3 c = step(vec3(0.0031308), linearRGB);
    return vec4(mix(a, b, c), linearRGB_in.a);
}

void main() {
    o_Target = encodeSRGB(vec4(v_color, 1.0));
}
"#;

fn setup(
    mut commands: Commands,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MyMaterialWithVertexColorSupport>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // Create a new shader pipeline
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind MyMaterialWithVertexColorSupport resources to our shader
    render_graph.add_system_node(
        "my_material_with_vertex_color_support",
        AssetRenderResourcesNode::<MyMaterialWithVertexColorSupport>::new(true),
    );

    // Add a Render Graph edge connecting our new "my_material" node to the main pass node. This ensures "my_material" runs before the main pass
    render_graph
        .add_node_edge(
            "my_material_with_vertex_color_support",
            base::node::MAIN_PASS,
        )
        .unwrap();

    // Create a new material
    let material = materials.add(MyMaterialWithVertexColorSupport {});

    // create a generic cube
    let mut cube_with_vertex_colors = Mesh::from(shape::Cube { size: 2.0 });

    // insert our custom color attribute with some nice colors!
    cube_with_vertex_colors.set_attribute(
        // name of the attribute
        "Vertex_Color",
        // the vertex attributes, represented by `VertexAttributeValues`
        // NOTE: the attribute count has to be consistent across all attributes, otherwise bevy will panic.
        VertexAttributeValues::from(vec![
            // top
            [0.79, 0.73, 0.07],
            [0.74, 0.14, 0.29],
            [0.08, 0.55, 0.74],
            [0.20, 0.27, 0.29],
            // bottom
            [0.79, 0.73, 0.07],
            [0.74, 0.14, 0.29],
            [0.08, 0.55, 0.74],
            [0.20, 0.27, 0.29],
            // right
            [0.79, 0.73, 0.07],
            [0.74, 0.14, 0.29],
            [0.08, 0.55, 0.74],
            [0.20, 0.27, 0.29],
            // left
            [0.79, 0.73, 0.07],
            [0.74, 0.14, 0.29],
            [0.08, 0.55, 0.74],
            [0.20, 0.27, 0.29],
            // front
            [0.79, 0.73, 0.07],
            [0.74, 0.14, 0.29],
            [0.08, 0.55, 0.74],
            [0.20, 0.27, 0.29],
            // back
            [0.79, 0.73, 0.07],
            [0.74, 0.14, 0.29],
            [0.08, 0.55, 0.74],
            [0.20, 0.27, 0.29],
        ]),
    );

    // Setup our world
    commands
        // cube
        .spawn_bundle(MeshBundle {
            mesh: meshes.add(cube_with_vertex_colors), // use our cube with vertex colors
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert(material);
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(Vec3::new(3.0, 5.0, -8.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..Default::default()
    });
}
