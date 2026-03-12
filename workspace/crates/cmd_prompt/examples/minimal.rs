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
            Node {
                width: vw(100),
                height: vh(100),
                ..Default::default()
            },
            VtUi::new(term_id),
        ));
        commands.write_message(TermMsg::write(term_id, "hello\nhere are multiple lines\n"));
        commands
            .write_message(TermMsg::write(term_id, "This is a really long line! It should be wrapping. Just checking :) How are you doing today? I'm doing pretty good myself.\n"));

        // commands.spawn((
        //     Node {width: px(500), height: px(500), ..Default::default()},
        //     TextLayout::new_with_no_wrap(),
        //     Text::new(""),
        //     BackgroundColor(Color::BLACK),
        //     TextColor(Color::WHITE),
        //     ZIndex(1000),
        //     children![(TextSpan::new("Absolute layout mode Test!")),(TextSpan::new(" SAME LINE\n")),(TextSpan::new("LINE 2\n")), (TextSpan::new("LINE 3 GO!!!!"))]
        // ));
    });
    app.add_systems(
        PostUpdate,
        |mut commands: Commands, mut ran: Local<bool>| {
            if *ran {
                // commands.write_message(AppExit::Success);
            } else {
                *ran = true;
            }
        },
    );
    app.run();
}
