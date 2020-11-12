use crate::renderer::{WebGL2RenderContext, WebGL2RenderResourceContext};
use bevy::ecs::{Resources, World};
use bevy::render::{
    render_graph::{
        DependentNodeStager, Edge, NodeId, RenderGraph, RenderGraphStager, ResourceSlots,
    },
    renderer::RenderResourceContext,
};
use std::sync::Arc;

use bevy::utils::HashMap;
use parking_lot::RwLock;
use std::cell::{Ref, RefCell};

#[derive(Default)]
pub struct Device {
    context: RefCell<Option<web_sys::WebGl2RenderingContext>>,
}

impl Device {
    pub fn get_context(&self) -> std::cell::Ref<web_sys::WebGl2RenderingContext> {
        return Ref::map(self.context.borrow(), |t| {
            t.as_ref().expect("webgl context is set")
        });
    }

    pub fn set_context(&self, context: web_sys::WebGl2RenderingContext) {
        *self.context.borrow_mut() = Some(context);
    }
}

impl std::fmt::Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Device: {:?}", self.context.borrow()))
    }
}

unsafe impl Send for Device {}
unsafe impl Sync for Device {}

#[derive(Default)]
pub struct WebGL2Renderer {
    pub device: Arc<Device>,
}

impl WebGL2Renderer {
    pub fn run_graph(&mut self, world: &mut World, resources: &mut Resources) {
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        // stage nodes
        let mut stager = DependentNodeStager::loose_grouping();
        let stages = stager.get_stages(&render_graph).unwrap();
        let mut borrowed = stages.borrow(&mut render_graph);
        let render_resource_context = resources.get_mut::<Box<dyn RenderResourceContext>>();
        if render_resource_context.is_none() {
            return;
        }
        let mut render_resource_context = render_resource_context.unwrap();
        let render_resource_context = render_resource_context
            .downcast_mut::<WebGL2RenderResourceContext>()
            .unwrap();

        let node_outputs: Arc<RwLock<HashMap<NodeId, ResourceSlots>>> = Default::default();
        for stage in borrowed.iter_mut() {
            // TODO: sort jobs and slice by "amount of work" / weights
            // stage.jobs.sort_by_key(|j| j.node_states.len());

            let chunk_size = stage.jobs.len();
            for jobs_chunk in stage.jobs.chunks_mut(chunk_size) {
                let world = &*world;
                let render_resource_context = render_resource_context.clone();
                let node_outputs = node_outputs.clone();
                let mut render_context =
                    WebGL2RenderContext::new(self.device.clone(), render_resource_context);
                for job in jobs_chunk.iter_mut() {
                    for node_state in job.node_states.iter_mut() {
                        // bind inputs from connected node outputs
                        for (i, mut input_slot) in node_state.input_slots.iter_mut().enumerate() {
                            if let Edge::SlotEdge {
                                output_node,
                                output_index,
                                ..
                            } = node_state.edges.get_input_slot_edge(i).unwrap()
                            {
                                let node_outputs = node_outputs.read();
                                let outputs = if let Some(outputs) = node_outputs.get(output_node) {
                                    outputs
                                } else {
                                    panic!("node inputs not set")
                                };

                                let output_resource =
                                    outputs.get(*output_index).expect("output should be set");
                                input_slot.resource = Some(output_resource);
                            } else {
                                panic!("no edge connected to input")
                            }
                        }
                        node_state.node.update(
                            world,
                            resources,
                            &mut render_context,
                            &node_state.input_slots,
                            &mut node_state.output_slots,
                        );
                        node_outputs
                            .write()
                            .insert(node_state.id, node_state.output_slots.clone());
                    }
                }
            }
        }
    }

    pub fn update(&mut self, world: &mut World, resources: &mut Resources) {
        self.run_graph(world, resources);
        if let Some(render_resource_context) = resources.get::<Box<dyn RenderResourceContext>>() {
            render_resource_context.drop_all_swap_chain_textures();
            render_resource_context.clear_bind_groups();
        }
    }
}
