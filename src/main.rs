//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use avian3d::prelude::*;
use bevy::{
    log::{LogPlugin, tracing_subscriber::EnvFilter},
    prelude::*,
};
use q_player::prelude::*;

#[cfg(feature = "inspector")]
mod inspector;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[cfg(feature = "inspector")]
    Inspector,
    MainMenu,
    #[default]
    LoadingWorld,
    MainGame,
}

#[bevy_main]
fn main() -> AppExit {
    let mut app = App::new();

    // use RUST_LOG
    app.add_plugins((
        DefaultPlugins.set(LogPlugin {
            filter: EnvFilter::from_default_env().to_string(),
            ..Default::default()
        }),
        q_worldgen::WorldgenPlugin,
        q_player::PlayerPlugin,
        PhysicsPlugins::default(),
    ))
    .insert_state(GameState::default());

    #[cfg(feature = "inspector")]
    let _ = {
        app.add_plugins(inspector::InspectorIntegrationPlugin)
            .add_systems(
                OnEnter(GameState::Inspector),
                |mut player_state: ResMut<NextState<PlayerState>>| {
                    player_state.set(PlayerState::Disabled)
                },
            )
    };
    #[cfg(not(feature = "inspector"))]
    let _ = {
        app.add_systems(
            Startup,
            |mut player_state: ResMut<NextState<PlayerState>>| {
                player_state.set(PlayerState::Enabled)
            },
        )
    };

    app.run()
}
