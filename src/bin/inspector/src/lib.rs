// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod scene;
pub mod service;
pub mod ui;

pub mod prelude {
    pub use crate::scene::*;
    pub use crate::service::*;
    pub use crate::ui::*;
}

use bevy::{
    app::PluginGroupBuilder, gizmos::GizmoPlugin, input::InputPlugin,
    pbr::wireframe::WireframePlugin, picking::PickingPlugin, prelude::*, sprite::SpritePlugin,
    state::app::StatesPlugin,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use prelude::*;
use q_tasks::TaskPlugin;
use q_utils::plugin_deps;

use crate::service::InspectorStatePlugin;

pub struct InspectorPluginGroup;
impl PluginGroup for InspectorPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(crate::scene::ScenePlugin)
            .add(InspectorStatePlugin)
            .add(UiPlugin)
            .build()
    }
}

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        plugin_deps!(
            app,
            TaskPlugin,
            DefaultInspectorConfigPlugin,
            (WireframePlugin, WireframePlugin::default()),
            (
                EguiPlugin,
                EguiPlugin {
                    enable_multipass_for_primary_context: false
                }
            ),
            SpritePlugin,
            (PickingPlugin, PickingPlugin::default()),
            StatesPlugin,
            InputPlugin,
            GizmoPlugin,
        );
        // ensure plugins are inserted in order
        app.add_plugins(InspectorPluginGroup)
            .add_systems(Startup, || debug!("STARTUP"))
            .add_systems(Update, || debug_once!("UPDATE"));
    }
}
