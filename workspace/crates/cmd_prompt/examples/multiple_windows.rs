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
                display: Display::Grid,
                grid_template_columns: vec![RepeatedGridTrack::percent(2, 50.)],
                grid_template_rows: vec![RepeatedGridTrack::percent(2, 50.)],
                width: vw(100),
                height: vh(100),
                ..Default::default()
            },
            children![
                (
                    TextFont {
                        font_size: 12.,
                        ..Default::default()
                    },
                    BackgroundColor(Color::BLACK),
                    TextColor(Color::WHITE),
                    TerminalWindow(term_id),
                ),
                (
                    TextFont {
                        font_size: 16.,
                        ..Default::default()
                    },
                    BackgroundColor(Color::WHITE),
                    TextColor(Color::BLACK),
                    TerminalWindow(term_id),
                ),
                (
                    TextFont {
                        font_size: 20.,
                        ..Default::default()
                    },
                    BackgroundColor(Color::WHITE),
                    TextColor(Color::BLACK),
                    TerminalWindow(term_id),
                ),
                (
                    TextFont {
                        font_size: 24.,
                        ..Default::default()
                    },
                    BackgroundColor(Color::BLACK),
                    TextColor(Color::WHITE),
                    TerminalWindow(term_id),
                ),
            ],
        ));
        for i in 0..100 {
            commands.write_message(TerminalMessage::writeln(term_id, i));
        }
    });
    app.run();
}
