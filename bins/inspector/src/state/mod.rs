// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_egui::EguiContextPass;
use q_player::cam::CamService;
use q_service::{
    lifecycle::events::{DisableService, EnableService},
    prelude::*,
};

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

#[derive(ServiceData, Debug, Default, Clone, PartialEq)]
pub struct InspectorServiceData {
    pub game_running: bool,
}
#[derive(ServiceError, thiserror::Error, Debug, PartialEq, Clone, Copy)]
pub enum InspectorServiceErr {}

service!(InspectorService, InspectorServiceData, InspectorServiceErr);

pub struct InspectorStatePlugin;
impl Plugin for InspectorStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorSettings>()
            .add_service(
                InspectorService::default_spec().is_startup(true).with_deps(
                    vec![
                        CamService::handle().into(),
                        GizmosService::handle().into(),
                    ],
                ),
            )
            .register_type::<InspectorSettings>()
            .add_observer(set_cam_state)
            .configure_sets(
                EguiContextPass,
                UiSystems.run_if(not(service_initializing(
                    InspectorService::handle(),
                )
                .or(service_uninitialized(InspectorService::handle())))),
            );
    }
}

fn set_cam_state(
    trigger: Trigger<InspectorServiceStateChange>,
    mut commands: Commands,
) {
    info!("set_cam_state");
    let (old_state, new_state) = &trigger.event().0;
    match (old_state, new_state) {
        (ServiceState::Initializing, _) => {}
        (_, ServiceState::Enabled) => {
            commands.trigger(EnableService(CamService::handle()));
        }
        (_, ServiceState::Disabled | ServiceState::Failed(_)) => {
            commands.trigger(DisableService(CamService::handle()));
        }
        _ => {}
    }
}
