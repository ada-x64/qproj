use bevy::{prelude::*, text::LineHeight, window::WindowResolution};
use q_cmd_prompt::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resizable: true,
                resolution: WindowResolution::new(800, 600),
                ..Default::default()
            }),
            ..Default::default()
        }),
        TerminalPlugin,
    ));
    app.add_plugins((
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
    ));
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(Camera2d);
        let term_id = commands.spawn(Terminal).id();
        commands.spawn((
            TextFont {
                font_size: 12.,
                ..Default::default()
            },
            BackgroundColor(Color::BLACK),
            TextColor(Color::WHITE),
            TerminalWindow(term_id),
            Node {
                width: percent(100),
                height: percent(100),
                ..Default::default()
            },
        ));
        for i in 0..100 {
            commands.write_message(TerminalMessage::writeln(term_id, i));
        }
        commands
            .write_message(TerminalMessage::writeln(term_id, "This is a really long line! It should be wrapping. Just checking :) How are you doing today? I'm doing pretty good myself."));
    });
    app.run();
}
