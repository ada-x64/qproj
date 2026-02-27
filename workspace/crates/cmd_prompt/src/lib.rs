mod data;
mod messages;
mod plugin;
mod systems;
pub mod prelude {
    pub use super::data::*;
    pub use super::messages::*;
    pub use super::plugin::*;
    pub use super::systems::*;
    pub use bevy::prelude::*;
    #[cfg(test)]
    pub use q_test_harness::prelude::*;
    pub use tiny_bail::prelude::*;

    #[cfg(test)]
    pub fn test_harness(app: &mut App) {
        app.add_plugins((
            TestRunnerPlugin {
                executor_kind: bevy::ecs::schedule::ExecutorKind::MultiThreaded,
                ..Default::default()
            },
            TerminalPlugin,
        ));
        app.insert_resource(TestRunnerTimeout(1.));
    }
}
