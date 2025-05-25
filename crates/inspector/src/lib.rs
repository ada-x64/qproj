//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---

pub(crate) mod gizmos;
pub(crate) mod inspector_cam;
pub(crate) mod state;
pub(crate) mod tabs;

pub mod prelude {
    pub use crate::gizmos::*;
    pub use crate::inspector_cam::*;
    pub use crate::state::*;
    pub use crate::tabs::*;
}

use bevy::{pbr::wireframe::WireframePlugin, prelude::*};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use prelude::*;
pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let _ = {
            app.add_plugins((
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
