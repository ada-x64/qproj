// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::scene::{
    gizmos::GizmosPlugin, inspector_cam::InspectorCamPlugin, serialize::SceneSerializePlugin,
};

pub mod gizmos;
pub mod inspector_cam;
pub mod serialize;

struct ScenePluginGroup;
impl PluginGroup for ScenePluginGroup {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(InspectorCamPlugin)
            .add(GizmosPlugin)
            .add(SceneSerializePlugin)
    }
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::scene::ScenePlugin>() {
            app.add_plugins(bevy::scene::ScenePlugin);
        }
        app.add_plugins(ScenePluginGroup)
            .add_systems(Startup, |mut commands: Commands| {
                debug!("Spawning scene root.");
                commands.spawn((Name::new("Scene Root"), DynamicSceneRoot::default()));
            });
    }
}
