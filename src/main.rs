//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use avian3d::prelude::*;
use bevy::{
    log::{LogPlugin, tracing_subscriber::EnvFilter},
    prelude::*,
};
use q_player::PlayerSet;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[cfg(feature = "inspector")]
    #[cfg_attr(feature = "inspector", default)]
    Inspector,
    MainMenu,
    #[cfg_attr(not(feature = "inspector"), default)]
    LoadingWorld,
    MainGame,
}

#[bevy_main]
fn main() {
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

    #[cfg(feature = "debug")]
    {
        use q_debug::{DebugLevel, DebugPlugin};
        let level = std::env::var("DEBUG_LEVEL").unwrap_or_default();
        let debug_level = DebugLevel(level.parse().unwrap_or_default());
        debug!("DEBUG_LEVEL = {debug_level:?}");
        app.add_plugins(DebugPlugin {
            debug_level,
            wireframes: true,
        });
    }

    #[cfg(feature = "inspector")]
    {
        fn enable_ui(mut state: ResMut<NextState<InspectorState>>) {
            info!("ENABLING INSPECTOR UI");
            state.set(InspectorState::Enabled);
        }
        use q_inspector::state::InspectorState;
        app.add_plugins((q_inspector::InspectorPlugin,))
            .add_systems(Startup, enable_ui)
            .add_systems(OnEnter(InspectorState::Disabled), q_player::spawn)
            .configure_sets(
                Update,
                PlayerSet::Active.run_if(in_state(InspectorState::Disabled)),
            );
    }

    app.configure_sets(
        Update,
        PlayerSet::Active.run_if(in_state(GameState::MainGame)),
    );

    app.run();
}
