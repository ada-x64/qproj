//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use q_utils::boolish_states;

use crate::{
    prelude::*,
    scene::{gizmos::GizmosState, inspector_cam::InspectorCamState},
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

boolish_states!(InspectorState, GameViewState);

// Plugin /////////////////////////////////////////////////////////////////////

/// Controls state for the entire application.
pub struct InspectorStatePlugin;
impl InspectorStatePlugin {
    fn init(
        settings: Res<InspectorSettings>,
        mut game_view: ResMut<NextState<GameViewState>>,
        mut cam: ResMut<NextState<InspectorCamState>>,
        mut gizmos: ResMut<NextState<GizmosState>>,
    ) {
        game_view.set(GameViewState::Disabled);
        cam.set(InspectorCamState::Enabled);
        gizmos.set(settings.enable_gizmo_overlay.into())
    }

    /// TODO: Physics should be handled _outside_ the inspector
    fn pause_time(
        // mut time: ResMut<Time<Physics>>,
        mut cam: ResMut<NextState<InspectorCamState>>,
    ) {
        // time.pause();
        cam.set(true.into())
    }

    /// TODO: Physics should be handled _outside_ the inspector
    fn unpause_time(
        // mut time: ResMut<Time<Physics>>,
        mut cam: ResMut<NextState<InspectorCamState>>,
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
            .add_systems(OnExit(InspectorState::Init), Self::init)
            .add_systems(OnEnter(GameViewState::Disabled), Self::pause_time)
            .add_systems(OnEnter(GameViewState::Enabled), Self::unpause_time)
            .configure_sets(
                PostUpdate,
                UiSystems.run_if(
                    in_state(InspectorState::Enabled)
                        .or(in_state(InspectorState::Disabled)),
                ),
            );
    }
}
