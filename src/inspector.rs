use bevy::prelude::*;
use q_inspector::{cam::InspectorCam, state::InspectorState};
use q_player::{PlayerCam, PlayerState};

pub struct InspectorIntegrationPlugin;

fn enable_ui(mut state: ResMut<NextState<InspectorState>>) {
    info!("ENABLING INSPECTOR UI");
    state.set(InspectorState::Enabled);
}
fn transition_state(
    inspector_state: Res<State<InspectorState>>,
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
    let enabled = matches!(inspector_state.get(), InspectorState::Enabled);
    player_cam.is_active = !enabled;
    inspector_cam.is_active = enabled;
    player_state.set(if enabled {
        PlayerState::Inactive
    } else {
        PlayerState::Active
    });
}
impl Plugin for InspectorIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((q_inspector::InspectorPlugin,))
            .add_systems(Startup, enable_ui)
            .add_systems(OnEnter(InspectorState::Disabled), transition_state)
            .add_systems(OnEnter(InspectorState::Enabled), transition_state);
    }
}
