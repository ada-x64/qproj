//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use avian3d::prelude::*;
use bevy::prelude::*;
use q_utils::boolish_states;

use crate::prelude::*;

// Resources //////////////////////////////////////////////////////////////////
#[derive(Clone, PartialEq, Eq, Hash, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct InspectorSettings {
    pub switch_cams: bool,
}
impl Default for InspectorSettings {
    fn default() -> Self {
        Self { switch_cams: true }
    }
}

boolish_states!(InspectorState, GameViewState);

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UISet;

// Plugin /////////////////////////////////////////////////////////////////////
pub struct InspectorStatePlugin;
impl InspectorStatePlugin {
    fn init(
        mut game_view: ResMut<NextState<GameViewState>>,
        mut cam: ResMut<NextState<InspectorCamState>>,
    ) {
        game_view.set(GameViewState::Disabled);
        cam.set(InspectorCamState::Enabled);
    }

    /// TODO: Physics should be handled _outside_ the inspector
    fn pause_time(
        mut time: ResMut<Time<Physics>>,
        mut cam: ResMut<NextState<InspectorCamState>>,
    ) {
        time.pause();
        cam.set(true.into())
    }

    /// TODO: Physics should be handled _outside_ the inspector
    fn unpause_time(
        mut time: ResMut<Time<Physics>>,
        mut cam: ResMut<NextState<InspectorCamState>>,
        settings: Res<InspectorSettings>,
    ) {
        time.unpause();
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
                UISet.run_if(
                    in_state(InspectorState::Enabled)
                        .or(in_state(InspectorState::Disabled)),
                ),
            );
    }
}
