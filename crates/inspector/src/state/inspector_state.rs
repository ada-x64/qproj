//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use avian3d::prelude::*;
use bevy::prelude::*;
use easy_ext::ext;
use q_utils::boolish_states;

use crate::components::cam::InspectorCam;

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

boolish_states!(InspectorState, GameViewState, CamState);

// Systems ////////////////////////////////////////////////////////////////////

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UISet;

fn init(
    mut game_view: ResMut<NextState<GameViewState>>,
    mut cam: ResMut<NextState<CamState>>,
) {
    game_view.set(GameViewState::Disabled);
    cam.set(CamState::Enabled);
}

/// TODO: Physics should be handled _outside_ the inspector
fn pause_time(
    mut time: ResMut<Time<Physics>>,
    mut cam: ResMut<NextState<CamState>>,
) {
    time.pause();
    cam.set(true.into())
}

/// TODO: Physics should be handled _outside_ the inspector
fn unpause_time(
    mut time: ResMut<Time<Physics>>,
    mut cam: ResMut<NextState<CamState>>,
    settings: Res<InspectorSettings>,
) {
    time.unpause();
    if settings.switch_cams {
        cam.set(false.into())
    }
}

fn set_cam_active<const VAL: bool>(
    mut cam: Single<&mut Camera, With<InspectorCam>>,
) {
    cam.is_active = VAL;
}

// Setup //////////////////////////////////////////////////////////////////////
#[ext(SetupInspectorState)]
pub impl App {
    fn setup_inspector_state(&mut self) -> &mut Self {
        self.setup_boolish_states()
            .init_resource::<InspectorSettings>()
            .register_type::<InspectorSettings>()
            .add_systems(OnExit(InspectorState::Init), init)
            .add_systems(OnEnter(CamState::Disabled), set_cam_active::<false>)
            .add_systems(OnEnter(CamState::Enabled), set_cam_active::<true>)
            .add_systems(OnEnter(GameViewState::Disabled), pause_time)
            .add_systems(OnEnter(GameViewState::Enabled), unpause_time)
            .configure_sets(
                PostUpdate,
                UISet.run_if(
                    in_state(InspectorState::Enabled)
                        .or(in_state(InspectorState::Disabled)),
                ),
            )
    }
}
