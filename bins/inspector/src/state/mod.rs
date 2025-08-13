// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_egui::EguiContextPass;
use q_player::cam::CamService;
use q_service::prelude::*;

use crate::{prelude::*, scene::gizmos::GizmosService};

// Resources //////////////////////////////////////////////////////////////////
#[derive(Clone, PartialEq, Eq, Hash, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct InspectorSettings {
    pub switch_cams: bool,
    pub enable_gizmo_overlay: bool,
}
impl Default for InspectorSettings {
    fn default() -> Self {
        Self {
            switch_cams: true,
            enable_gizmo_overlay: true,
        }
    }
}

// Plugin /////////////////////////////////////////////////////////////////////

#[derive(Resource, Debug, Default, Clone, PartialEq)]
pub struct InspectorService {
    pub game_running: bool,
}
impl Service for InspectorService {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            .is_startup(true)
            .add_dep::<GizmosService>()
            .add_dep::<CamService>();
    }
}

pub struct InspectorStatePlugin;
impl Plugin for InspectorStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorSettings>()
            .register_type::<InspectorSettings>()
            .register_service::<InspectorService>()
            .configure_sets(
                EguiContextPass,
                UiSystems.run_if(service_up::<InspectorService>()),
            );
    }
}
