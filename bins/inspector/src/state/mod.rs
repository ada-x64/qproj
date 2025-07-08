// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_egui::EguiContextPass;
use q_utils::boolish_states;

use crate::{
    prelude::*,
    scene::{
        gizmos::SetGizmosEnabled,
        inspector_cam::{InspectorCamStates, SetInspectorCamEnabled},
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

boolish_states!(Inspector, GameView);

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

impl Plugin for InspectorStatePlugin {
    fn build(&self, app: &mut App) {
        app.setup_boolish_states()
            .init_resource::<InspectorSettings>()
            .register_type::<InspectorSettings>()
            .add_observer(trigger_init)
            .add_systems(OnEnter(GameViewStates::Disabled), Self::pause_time)
            .add_systems(OnEnter(GameViewStates::Enabled), Self::unpause_time)
            .configure_sets(
                EguiContextPass,
                UiSystems.run_if(not(in_state(InspectorStates::Init))),
            );
    }
}

#[derive(Resource, Debug)]
struct InitializedServices {
    game_view: bool,
    inspector_cam: bool,
    gizmos: bool,
}

fn trigger_init(
    _trigger: Trigger<InitInspector>,
    settings: Res<InspectorSettings>,
    mut commands: Commands,
) {
    commands.trigger(SetGameViewEnabled(false));
    commands.trigger(SetInspectorCamEnabled(true));
    commands.trigger(SetGizmosEnabled(settings.enable_gizmo_overlay));
    // wait for all the other initialization events
}
