pub mod converters;
mod default_plugins;
pub mod renderer;
mod webgl2_render_pass;
mod webgl2_renderer;
mod webgl2_resources;
use crate::renderer::WebGL2RenderResourceContext;
use bevy::app::{prelude::*, Events};
use bevy::window::{WindowCreated, Windows};
pub use default_plugins::*;
use std::sync::Arc;
pub use webgl2_render_pass::*;
pub use webgl2_renderer::*;
pub use webgl2_resources::*;

use bevy::asset::{Assets, HandleUntyped};
use bevy::ecs::prelude::*;
use bevy::ecs::{
    schedule::{StageLabel, SystemStage},
    world::World,
};
use bevy::reflect::TypeUuid;
use bevy::render::{
    pipeline::PipelineDescriptor,
    renderer::{shared_buffers_update_system, RenderResourceContext, SharedBuffers},
    shader::{Shader, ShaderStage},
    RenderStage,
};

pub const SPRITE_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 2785347840338765446);

pub const SPRITE_SHEET_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 9016885805180281612);

pub const PBR_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 13148362314012771389);

pub const UI_PIPELINE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(PipelineDescriptor::TYPE_UUID, 3234320022263993878);
#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum WebGL2Stage {
    PreRenderResource,
}

#[derive(Default)]
pub struct WebGL2Plugin;

impl Plugin for WebGL2Plugin {
    fn build(&self, app: &mut App) {
        {
            let world = &mut app.world;
            let cell = world.cell();
            let pipelines = cell
                .get_resource_mut::<Assets<PipelineDescriptor>>()
                .unwrap();
            let mut shaders = cell.get_resource_mut::<Assets<Shader>>().unwrap();

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
                    PBR_PIPELINE_HANDLE,
                    include_str!("shaders/pbr.vert"),
                    include_str!("shaders/pbr.frag"),
                ),
                (
                    UI_PIPELINE_HANDLE,
                    include_str!("shaders/ui.vert"),
                    include_str!("shaders/ui.frag"),
                ),
            ];

            for (pipeline_handle, vert_source, frag_source) in shader_overrides {
                if let Some(pipeline) = pipelines.get(pipeline_handle) {
                    let _ = shaders.set(
                        &pipeline.shader_stages.vertex,
                        Shader::from_glsl(ShaderStage::Vertex, vert_source),
                    );
                    if let Some(frag_handle) = &pipeline.shader_stages.fragment {
                        let _ = shaders.set(
                            frag_handle,
                            Shader::from_glsl(ShaderStage::Fragment, frag_source),
                        );
                    }
                }
            }
        }
        let world = &mut app.world;
        let render_system = webgl2_render_system(world);
        let handle_events_system = webgl2_handle_window_created_events_system();
        app.add_stage_before(
            RenderStage::RenderResource,
            WebGL2Stage::PreRenderResource,
            SystemStage::parallel(),
        )
        .add_system_to_stage(
            WebGL2Stage::PreRenderResource,
            handle_events_system.exclusive_system(),
        )
        .add_system_to_stage(RenderStage::Render, render_system.exclusive_system())
        .add_system_to_stage(
            RenderStage::PostRender,
            shared_buffers_update_system.system(),
        );
    }
}

pub fn webgl2_handle_window_created_events_system() -> impl FnMut(&mut World) {
    let events = Events::<WindowCreated>::default();
    let mut window_created_event_reader = events.get_reader();

    move |world| {
        let events = {
            let window_created_events = world.get_resource::<Events<WindowCreated>>().unwrap();
            window_created_event_reader
                .iter(&window_created_events)
                .cloned()
                .collect::<Vec<_>>()
        };

        for window_created_event in events {
            let window_id = {
                let windows = world.get_resource::<Windows>().unwrap();
                let window = windows
                    .get(window_created_event.id)
                    .expect("Received window created event for non-existent window");
                window.id()
            };
            let render_resource_context = {
                let device = &*world.get_resource::<Arc<Device>>().unwrap();
                let winit_windows = world.get_resource::<bevy::winit::WinitWindows>().unwrap();
                let winit_window = winit_windows.get_window(window_id).unwrap();
                let mut render_resource_context = WebGL2RenderResourceContext::new(device.clone());
                render_resource_context.initialize(&winit_window);
                render_resource_context
            };
            world.insert_resource::<Box<dyn RenderResourceContext>>(Box::new(
                render_resource_context,
            ));
            //resources.insert(SharedBuffers::new(Box::new(render_resource_context)));
            world.insert_resource(SharedBuffers::new(4096));
        }
    }
}

pub fn webgl2_render_system(world: &mut World) -> impl FnMut(&mut World) {
    let mut webgl2_renderer = WebGL2Renderer::default();
    let device = webgl2_renderer.device.clone();
    world.insert_resource(device);
    move |world| {
        webgl2_renderer.update(world);
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
