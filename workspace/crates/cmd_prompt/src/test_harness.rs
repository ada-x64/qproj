use bevy::{
    image::TextureAtlasPlugin, input::InputPlugin, render::texture::TexturePlugin,
    text::TextPlugin, ui::UiPlugin,
};
use q_test_harness::{TestRunnerPlugin, TestRunnerTimeout};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
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
        ConsolePlugin,
    ));
    app.insert_resource(TestRunnerTimeout(1.));
}
