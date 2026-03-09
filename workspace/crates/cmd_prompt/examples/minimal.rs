use bevy::{prelude::*, window::WindowResolution};
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
        let window_id = commands.spawn((
            TerminalWindow(term_id),
            Node {
                width: percent(100),
                height: percent(100),
                ..Default::default()
            },
        )).id();
        for i in 0..100 {
            commands.write_message(TermMsg::writeln(window_id, i));
        }
        commands
            .write_message(TermMsg::writeln(window_id, "This is a really long line! It should be wrapping. Just checking :) How are you doing today? I'm doing pretty good myself."));
    });
    app.run();
}
