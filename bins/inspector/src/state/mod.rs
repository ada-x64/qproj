// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_egui::EguiContextPass;
use q_utils::service;

use crate::{
    prelude::*,
    scene::{
        gizmos::{EnableGizmos, GizmosInitialized, InitGizmos},
        inspector_cam::{InitInspectorCam, InspectorCamStates},
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

service!(GameView);
service!(Inspector, init_inspector);
// Wait for all subservices to initialize.
fn init_inspector(_trigger: Trigger<InitInspector>, mut commands: Commands) {
    commands.trigger(InitGameView);
    commands.trigger(InitInspectorCam);
}
impl Plugin for InspectorStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InspectorServicePlugin, GameViewServicePlugin))
            .init_resource::<InspectorSettings>()
            .register_type::<InspectorSettings>()
            .init_resource::<ServiceStatus>()
            .register_type::<ServiceStatus>()
            .add_observer(on_gameview_initialized)
            .add_observer(on_gizmos_initialized)
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
    mut commands: Commands,
    settings: Res<InspectorSettings>,
) {
    services.gizmos = Some(trigger.0.clone());
    commands.trigger(EnableGizmos(settings.enable_gizmo_overlay));
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
