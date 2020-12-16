mod default_plugins;
pub mod renderer;
mod webgl2_render_pass;
mod webgl2_renderer;
mod webgl2_resources;
use crate::renderer::WebGL2RenderResourceContext;
use bevy::app::prelude::*;
use bevy::window::{WindowCreated, Windows};
pub use default_plugins::*;
use std::sync::Arc;
pub use webgl2_render_pass::*;
pub use webgl2_renderer::*;
pub use webgl2_resources::*;

use bevy::asset::{Assets, HandleUntyped};
use bevy::ecs::prelude::*;
use bevy::ecs::{Resources, SystemStage, World};
use bevy::reflect::TypeUuid;
use bevy::render::{
    pipeline::PipelineDescriptor,
    renderer::{shared_buffers_update_system, RenderResourceContext, SharedBuffers},
    shader::{Shader, ShaderStage},
    stage::{RENDER, RENDER_RESOURCE},
};

pub const SPRITE_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 2785347840338765446);

pub const SPRITE_SHEET_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 9016885805180281612);

pub const FORWARD_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 13148362314012771389);

pub const UI_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 3234320022263993878);

#[derive(Default)]
pub struct WebGL2Plugin;

impl Plugin for WebGL2Plugin {
    fn build(&self, app: &mut AppBuilder) {
        let resources = app.resources_mut();
        {
            let pipelines = resources.get_mut::<Assets<PipelineDescriptor>>().unwrap();
            let mut shaders = resources.get_mut::<Assets<Shader>>().unwrap();

            let shader_overrides = vec![
                (
                    SPRITE_PIPELINE_HANDLE,
                    include_str!("shaders/sprite.vert"),
                    include_str!("shaders/sprite.frag"),
                ),
                (
                    SPRITE_SHEET_PIPELINE_HANDLE,
                    include_str!("shaders/sprite_sheet.vert"),
                    include_str!("shaders/sprite_sheet.frag"),
                ),
                (
                    FORWARD_PIPELINE_HANDLE,
                    include_str!("shaders/forward.vert"),
                    include_str!("shaders/forward.frag"),
                ),
                (
                    UI_PIPELINE_HANDLE,
                    include_str!("shaders/ui.vert"),
                    include_str!("shaders/ui.frag"),
                ),
            ];

            for (pipeline_handle, vert_source, frag_source) in shader_overrides {
                if let Some(pipeline) = pipelines.get(pipeline_handle) {
                    shaders.set(
                        &pipeline.shader_stages.vertex,
                        Shader::from_glsl(ShaderStage::Vertex, vert_source),
                    );
                    if let Some(frag_handle) = &pipeline.shader_stages.fragment {
                        shaders.set(
                            frag_handle,
                            Shader::from_glsl(ShaderStage::Fragment, frag_source),
                        );
                    }
                }
            }
        }
        let render_system = webgl2_render_system(resources);
        let handle_events_system = webgl2_handle_window_created_events_system();
        app.add_stage_before(
            RENDER_RESOURCE,
            "webgl2_pre_render_resource",
            SystemStage::parallel(),
        )
        .add_system_to_stage("webgl2_pre_render_resource", handle_events_system.system())
        .add_system_to_stage(RENDER, render_system.system())
        .add_system_to_stage(
            bevy::render::stage::POST_RENDER,
            shared_buffers_update_system.system(),
        );
    }
}

#[derive(Default)]
pub struct State {
    pub window_created_event_reader: EventReader<WindowCreated>,
}

pub fn webgl2_handle_window_created_events_system() -> impl FnMut(&mut World, &mut Resources) {
    let mut window_created_event_reader: EventReader<WindowCreated> = Default::default();
    move |_, resources| {
        let events = {
            let window_created_events = resources.get::<Events<WindowCreated>>().unwrap();
            window_created_event_reader
                .iter(&window_created_events)
                .cloned()
                .collect::<Vec<_>>()
        };

        for window_created_event in events {
            let window_id = {
                let windows = resources.get::<Windows>().unwrap();
                let window = windows
                    .get(window_created_event.id)
                    .expect("Received window created event for non-existent window");
                window.id()
            };
            let render_resource_context = {
                let device = &*resources.get::<Arc<Device>>().unwrap();
                let winit_windows = resources.get::<bevy::winit::WinitWindows>().unwrap();
                let winit_window = winit_windows.get_window(window_id).unwrap();
                let mut render_resource_context = WebGL2RenderResourceContext::new(device.clone());
                render_resource_context.initialize(&winit_window);
                render_resource_context
            };
            resources.insert::<Box<dyn RenderResourceContext>>(Box::new(render_resource_context));
            //resources.insert(SharedBuffers::new(Box::new(render_resource_context)));
            resources.insert(SharedBuffers::new(4096));
        }
    }
}

pub fn webgl2_render_system(resources: &mut Resources) -> impl FnMut(&mut World, &mut Resources) {
    let mut webgl2_renderer = WebGL2Renderer::default();
    let device = webgl2_renderer.device.clone();
    resources.insert(device);
    move |world, resources| {
        webgl2_renderer.update(world, resources);
    }
}

#[macro_export]
macro_rules! gl_call {
    ($device:ident . $func:ident ( $( $i:expr),* $(,)? ) ) => {
        {
            // trace!("gl call: {} {:?}", stringify!($func ( $( $i ),*)), ( $( $i ),*) );
            let result = $device . $func( $( $i ),* );
            result
        }
    };
}
