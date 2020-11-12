use bevy::app::{PluginGroup, PluginGroupBuilder};

pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        bevy::prelude::DefaultPlugins.build(group);
        group.add(crate::WebGL2Plugin::default());
    }
}
