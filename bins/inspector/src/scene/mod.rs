// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;

use crate::scene::{
    gizmos::GizmosPlugin, inspector_cam::InspectorCamPlugin, serialize::SceneSerializePlugin,
};

pub mod gizmos;
pub mod inspector_cam;
pub mod serialize;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::scene::ScenePlugin>() {
            app.add_plugins(bevy::scene::ScenePlugin);
        }
        app.add_plugins((InspectorCamPlugin, GizmosPlugin, SceneSerializePlugin))
            .add_systems(Startup, |mut commands: Commands| {
                debug!("Spawning scene root.");
                commands.spawn((Name::new("Scene Root"), DynamicSceneRoot::default()));
            });
    }
}
