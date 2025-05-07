//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---

pub(crate) mod components;
pub(crate) mod gizmos;
pub(crate) mod state;
pub(crate) mod tabs;

pub mod prelude {
    pub use crate::components::*;
    pub use crate::gizmos::*;
    pub use crate::state::*;
    pub use crate::tabs::*;
}

use bevy::prelude::*;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use components::SetupComponents;
use state::{SetupInspectorState, SetupUi};
use tabs::SetupTabs;
pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let _ = {
            app.add_plugins((
                DefaultInspectorConfigPlugin,
                bevy_egui::EguiPlugin,
            ))
            .setup_inspector_state()
            .setup_tabs()
            .setup_ui()
            .setup_components()
        };
    }
}
