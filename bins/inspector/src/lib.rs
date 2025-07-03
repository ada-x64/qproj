//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---

pub(crate) mod gizmos;
pub(crate) mod inspector_cam;
pub(crate) mod scene;
pub(crate) mod state;
pub(crate) mod tabs;
pub(crate) mod widgets;

pub mod prelude {
    pub use crate::gizmos::*;
    pub use crate::inspector_cam::*;
    pub use crate::state::*;
    pub use crate::tabs::*;
}

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use prelude::*;
use q_tasks::TaskPlugin;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let _ = {
            if !app.is_plugin_added::<TaskPlugin>() {
                app.add_plugins(TaskPlugin);
            }
            app.add_plugins((
                crate::scene::ScenePlugin,
                WireframePlugin::default(),
                DefaultInspectorConfigPlugin,
                bevy_egui::EguiPlugin {
                    enable_multipass_for_primary_context: false,
                },
                InspectorCamPlugin,
                InspectorStatePlugin,
                UiStatePlugin,
                GizmosPlugin,
            ))
        };
    }
}
