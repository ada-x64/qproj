//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use q_inspector::prelude::*;
use q_player::PlayerState;

pub struct InspectorIntegrationPlugin;

fn enable_ui(mut state: ResMut<NextState<InspectorState>>) {
    info!("ENABLING INSPECTOR UI");
    state.set(InspectorState::Enabled);
}
fn transition_state(
    active: Res<State<GameViewState>>,
    mut player_state: ResMut<NextState<PlayerState>>,
    mut player_cam_state: ResMut<NextState<q_player::CamState>>,
    settings: Res<InspectorSettings>,
) {
    let enabled = matches!(active.get(), GameViewState::Enabled);
    player_state.set(enabled.into());
    let cam_enabled = settings.switch_cams && enabled;
    player_cam_state.set(cam_enabled.into());
}

impl Plugin for InspectorIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((q_inspector::InspectorPlugin,))
            .add_systems(Startup, enable_ui)
            .add_systems(OnEnter(GameViewState::Disabled), transition_state)
            .add_systems(OnEnter(GameViewState::Enabled), transition_state)
            .add_systems(
                Update,
                transition_state.run_if(resource_changed::<InspectorSettings>),
            );
    }
}
