use avian3d::PhysicsPlugins;
use bevy::{app::AppLabel, prelude::*};
use q_player::prelude::PlayerState;

#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    MainMenu,
    #[default]
    LoadingWorld,
    MainGame,
}

#[derive(AppLabel, Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct GameLabel;

/// Plugin for the full game integration.
/// This needs to be a separate plugin so we can share it with the inspector.
pub struct GameAppPlugin {
    /// Determines whether the game is running as the main application.
    pub main_app: bool,
}
impl Default for GameAppPlugin {
    fn default() -> Self {
        Self { main_app: true }
    }
}

impl Plugin for GameAppPlugin {
    fn build(&self, app: &mut App) {
        let plugins = (
            q_worldgen::WorldgenPlugin,
            q_player::PlayerPlugin,
            q_tasks::TaskPlugin,
            PhysicsPlugins::default(),
        );
        if !self.main_app {
            let mut subapp = SubApp::new();
            subapp.add_plugins(plugins);
            app.insert_sub_app(GameLabel, subapp);
        } else {
            app.add_plugins(plugins);
        };
        // app.add_plugins()
        // .insert_state(GameState::default())
        // .add_systems(
        //     Startup,
        //     |mut player_state: ResMut<NextState<PlayerState>>| {
        //         player_state.set(PlayerState::Enabled)
        //     },
        // );
    }
}
