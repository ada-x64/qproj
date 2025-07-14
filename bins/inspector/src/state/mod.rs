// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_egui::EguiContextPass;
use q_service::{
    ServiceExt,
    prelude::{ServiceError, ServiceLabel},
    service,
};

use crate::{
    prelude::*,
    scene::{
        gizmos::{GIZMOS_SERVICE, GizmosInitialized, InitGizmos},
        inspector_cam::{
            INSPECTOR_CAM_SERVICE, InspectorCamInitialized, InspectorCamStates,
        },
    },
};

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

/// Controls state for the entire application.
pub struct InspectorStatePlugin;
impl InspectorStatePlugin {
    /// TODO: Physics should be handled _outside_ the inspector
    fn pause_time(
        // mut time: ResMut<Time<Physics>>,
        mut cam: ResMut<NextState<InspectorCamStates>>,
    ) {
        // time.pause();
        cam.set(true.into())
    }

    /// TODO: Physics should be handled _outside_ the inspector
    fn unpause_time(
        // mut time: ResMut<Time<Physics>>,
        mut cam: ResMut<NextState<InspectorCamStates>>,
        settings: Res<InspectorSettings>,
    ) {
        // time.unpause();
        if settings.switch_cams {
            cam.set(false.into())
        }
    }
}

#[derive(ServiceLabel, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Services {
    GameView,
    Inspector,
    InspectorCam,
    Gizmos,
}

#[derive(ServiceError, thiserror::Error, Debug, PartialEq, Clone, Copy)]
pub enum GameViewErr {}
#[derive(ServiceError, thiserror::Error, Debug, PartialEq, Clone, Copy)]
pub enum InspectorServiceErr {}

// TODO: Specify service spec constant here.
// Make syntax more flexible. Use key-value pairs.
service!(GameView, Services, (), GameViewErr);
service!(Inspector, Services, (), InspectorServiceErr);

impl Plugin for InspectorStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectorSettings>()
            .add_service(GAME_VIEW_SERVICE_SPEC)
            .add_service(INSPECTOR_SERVICE_SPEC.is_startup(true).with_deps(
                vec![GAME_VIEW_SERVICE, INSPECTOR_CAM_SERVICE, GIZMOS_SERVICE],
            ))
            .register_type::<InspectorSettings>()
            .add_observer(on_gameview_initialized)
            .add_observer(on_gizmos_initialized)
            .add_observer(on_inspector_cam_initialized)
            .add_systems(OnEnter(GameViewStates::Disabled), Self::pause_time)
            .add_systems(OnEnter(GameViewStates::Enabled), Self::unpause_time)
            .configure_sets(
                EguiContextPass,
                UiSystems.run_if(not(in_state(InspectorStates::Initializing)
                    .or(in_state(InspectorStates::Uninitialized)))),
            );
    }
}

#[derive(Resource, Debug, Default, Reflect)]
struct ServiceStatus {
    game_view: Option<Result<bool, String>>,
    inspector_cam: Option<Result<bool, String>>,
    gizmos: Option<Result<bool, String>>,
}

//NB Gizmos must be triggered _after_ inspector cam is initialized
fn on_gameview_initialized(
    trigger: Trigger<GameViewInitialized>,
    mut services: ResMut<ServiceStatus>,
    mut commands: Commands,
) {
    services.game_view = Some(trigger.0.clone());
    commands.trigger(InitGizmos);
    check_if_done(services.as_ref(), commands);
}

fn on_gizmos_initialized(
    trigger: Trigger<GizmosInitialized>,
    mut services: ResMut<ServiceStatus>,
    commands: Commands,
) {
    services.gizmos = Some(trigger.0.clone());
    check_if_done(services.as_ref(), commands);
}

fn on_inspector_cam_initialized(
    trigger: Trigger<InspectorCamInitialized>,
    mut services: ResMut<ServiceStatus>,
    commands: Commands,
) {
    services.inspector_cam = Some(trigger.0.clone());
    check_if_done(services.as_ref(), commands);
}

fn check_if_done(services: &ServiceStatus, mut commands: Commands) {
    let ok = services.game_view.is_some()
        && services.inspector_cam.is_some()
        && services.gizmos.is_some();
    if ok {
        commands.trigger(InspectorInitialized(Ok(true)))
    }
}
