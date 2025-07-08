//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---

pub mod scene;
pub mod state;
pub mod ui;

pub mod prelude {
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
        };
    }
}
