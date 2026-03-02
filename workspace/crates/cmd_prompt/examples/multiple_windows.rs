use bevy::{prelude::*, window::WindowResolution};
use q_cmd_prompt::prelude::*;

const LOREM_IPSUM: &str = "Incidunt qui quas aut. Fugit aut laudantium omnis qui adipisci. Fuga recusandae sint ipsam. Reiciendis sint delectus labore quam sapiente doloremque. Repellendus laudantium et et rerum ut sed.";

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

        let children = (0..4)
            .map(|i| {
                let (foreground, background) = match i {
                    0 | 3 => (Color::BLACK, Color::WHITE),
                    _ => (Color::WHITE, Color::BLACK),
                };
                commands
                    .spawn((
                        TextFont {
                            font_size: 12. + i as f32 * 4.,
                            ..Default::default()
                        },
                        BackgroundColor(background),
                        TextColor(foreground),
                        TerminalWindow(term_id),
                        Node {
                            width: percent(100),
                            ..Default::default()
                        },
                    ))
                    .id()
            })
            .collect::<Vec<_>>();

        commands
            .spawn((Node {
                display: Display::Grid,
                grid_template_columns: vec![RepeatedGridTrack::percent(2, 50.)],
                grid_template_rows: vec![RepeatedGridTrack::percent(2, 50.)],
                width: vw(100),
                height: vh(100),
                ..Default::default()
            },))
            .add_children(&children);

        for i in 0..100 {
            // Although I'm only writing to the first child, they'll _all_ receive the same info,
            // because they are displaying the same `Terminal` entity
            commands.write_message(TerminalMessage::writeln(children[0], i));
        }
        let long_string = ["."].repeat(256).join("");
        commands.write_message(TerminalMessage::writeln(children[0], long_string));
        commands.write_message(TerminalMessage::writeln(children[0], LOREM_IPSUM));
    });
    app.run();
}
