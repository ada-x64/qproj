//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use q_inspector::state::{GameViewActive, InspectorEnabled, InspectorSettings};
use q_player::PlayerState;

pub struct InspectorIntegrationPlugin;

fn enable_ui(mut state: ResMut<NextState<InspectorEnabled>>) {
    info!("ENABLING INSPECTOR UI");
    state.set(InspectorEnabled::Enabled);
}
fn transition_state(
    active: Res<State<GameViewActive>>,
    mut player_state: ResMut<NextState<PlayerState>>,
    mut player_cam_state: ResMut<NextState<q_player::CamState>>,
    settings: Res<InspectorSettings>,
) {
    let enabled = matches!(active.get(), GameViewActive::Enabled);
    player_state.set(enabled.into());
    let cam_enabled = settings.switch_cams && enabled;
    player_cam_state.set(cam_enabled.into());
}

impl Plugin for InspectorIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((q_inspector::InspectorPlugin,))
            .add_systems(Startup, enable_ui)
            .add_systems(OnEnter(GameViewActive::Disabled), transition_state)
            .add_systems(OnEnter(GameViewActive::Enabled), transition_state)
            .add_systems(
                Update,
                transition_state.run_if(resource_changed::<InspectorSettings>),
            );
    }
}
