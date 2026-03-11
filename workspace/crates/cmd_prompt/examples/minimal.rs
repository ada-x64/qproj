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
        commands.spawn((
            VtUi::new(term_id),
            Node {
                width: vw(100),
                height: vh(100),
                ..Default::default()
            },
        ));
        commands.write_message(TermMsg::write(term_id, "hello\n"));
        // }
        // commands
        //     .write_message(TermMsg::write(term_id, "This is a really long line! It should be wrapping. Just checking :) How are you doing today? I'm doing pretty good myself.\n"));
    });
    app.add_systems(
        PostUpdate,
        |mut commands: Commands, mut ran: Local<bool>| {
            if *ran {
                commands.write_message(AppExit::Success);
            } else {
                *ran = true;
            }
        },
    );
    app.run();
}
