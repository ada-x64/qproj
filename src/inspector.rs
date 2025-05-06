//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use q_inspector::{
    cam::InspectorCam,
    state::{InspectorEnabled, InspectorSettings, PhysicsEnabled},
};
use q_player::{PlayerCam, PlayerState};

pub struct InspectorIntegrationPlugin;

fn enable_ui(mut state: ResMut<NextState<InspectorEnabled>>) {
    info!("ENABLING INSPECTOR UI");
    state.set(InspectorEnabled::Enabled);
}
fn transition_state(
    physics: Res<State<PhysicsEnabled>>,
    inspector_settings: Res<InspectorSettings>,
    mut player_state: ResMut<NextState<PlayerState>>,
    mut player_cam: Query<
        &mut Camera,
        (With<PlayerCam>, Without<InspectorCam>),
    >,
    mut inspector_cam: Query<
        &mut Camera,
        (With<InspectorCam>, Without<PlayerCam>),
    >,
) {
    let (Ok(mut player_cam), Ok(mut inspector_cam)) =
        (player_cam.get_single_mut(), inspector_cam.get_single_mut())
    else {
        return;
    };
    if inspector_settings.switch_cams {
        let enabled = matches!(physics.get(), PhysicsEnabled::Enabled);
        player_cam.is_active = !enabled;
        inspector_cam.is_active = enabled;
        player_state.set(if enabled {
            PlayerState::Inactive
        } else {
            PlayerState::Active
        });
    } else {
        player_cam.is_active = false;
    }
}

impl Plugin for InspectorIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((q_inspector::InspectorPlugin,))
            .add_systems(Startup, enable_ui)
            .add_systems(OnEnter(InspectorEnabled::Disabled), transition_state)
            .add_systems(OnEnter(InspectorEnabled::Enabled), transition_state)
            .add_systems(
                Update,
                transition_state.run_if(resource_changed::<InspectorSettings>),
            );
    }
}
