mod ansi;
mod data;
mod messages;
mod plugin;
mod systems;
pub mod prelude {
    pub use super::ansi::*;
    pub use super::data::*;
    pub use super::messages::*;
    pub use super::plugin::*;
    pub use super::systems::*;
    #[cfg(test)]
    pub use super::test::*;
    pub use crate::term_writeln;
    pub use bevy::prelude::*;
    pub use tiny_bail::prelude::*;
}
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use bevy::{
        image::TextureAtlasPlugin, input::InputPlugin, picking::PickingSettings,
        render::texture::TexturePlugin, text::TextPlugin, ui::UiPlugin,
    };
    pub use q_test_harness::prelude::*;
    pub fn test_harness(app: &mut App) {
        app.insert_resource(PickingSettings {
            is_window_picking_enabled: false,
            ..Default::default()
        });
        app.add_plugins((
            TestRunnerPlugin::default(),
            DefaultPickingPlugins,
            WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..Default::default()
            },
            InputPlugin,
            UiPlugin,
            TextPlugin,
            TextureAtlasPlugin,
            ImagePlugin::default(),
            TexturePlugin,
            TerminalPlugin,
        ));
        app.insert_resource(TestRunnerTimeout(1.));
    }
}
