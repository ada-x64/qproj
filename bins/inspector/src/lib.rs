// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod scene;
mod state;
mod ui;

pub mod prelude {
    pub use crate::scene::*;
    pub use crate::state::*;
    pub use crate::ui::*;
}

use bevy::{
    gizmos::GizmoPlugin, input::InputPlugin, pbr::wireframe::WireframePlugin,
    picking::PickingPlugin, prelude::*, sprite::SpritePlugin,
    state::app::StatesPlugin,
};
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use prelude::*;
use q_tasks::TaskPlugin;
use q_utils::plugin_deps;

use crate::state::InspectorStatePlugin;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let _ = {
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
            app.add_plugins((
                crate::scene::ScenePlugin,
                InspectorStatePlugin,
                UiPlugin,
            ))
            .add_systems(Startup, || debug!("STARTUP"))
            .add_systems(Update, || debug_once!("UPDATE"))
        };
    }
}
