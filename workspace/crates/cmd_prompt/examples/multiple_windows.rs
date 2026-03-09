use bevy::{
    color::palettes::css,
    input::{ButtonState, keyboard::KeyboardInput},
    picking::hover::HoverMap,
    prelude::*,
    window::WindowResolution,
};
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
    app.insert_resource(ShowEgui(true));
    app.add_plugins((
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::new()
            .run_if(resource_equals(ShowEgui(true))),
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
                    ))
                    .id()
            })
            .collect::<Vec<_>>();

        commands
            .spawn((
                Node {
                    display: Display::Grid,
                    grid_template_columns: vec![RepeatedGridTrack::percent(2, 50.)],
                    grid_template_rows: vec![RepeatedGridTrack::percent(2, 50.)],
                    width: vw(100),
                    height: vh(100),
                    ..Default::default()
                },
                ZIndex(-1000),
            ))
            .add_children(&children);

        for i in 0..100 {
            commands.write_message(TermMsg::writeln(term_id, i));
        }
        let long_string = ["."].repeat(256).join("");
        commands.write_message(TermMsg::writeln(term_id, long_string));
        commands.write_message(TermMsg::writeln(term_id, LOREM_IPSUM));
        commands.write_message(TermMsg::writeln_rich(
            term_id,
            term_writeln!(
                "This is some ",
                ("fancy", background = css::GREEN, color = css::RED),
                " text!"
            ),
        ));

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                display: Display::Flex,
                ..Default::default()
            },
            TextColor(css::RED.into()),
            Text::default(),
            DebugText,
            ZIndex(1000),
        ));
    });
    app.add_systems(Update, (update_debug_text, toggle_egui));
    app.run();
}

fn update_debug_text(q: Query<&mut Text, With<DebugText>>, hovered: Res<HoverMap>) {
    for mut text in q {
        text.0 = hovered
            .0
            .iter()
            .map(|(k, v)| {
                let mapped = v
                    .iter()
                    .map(|(k, v)| format!("(${k:?}) => ${v:#?}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{k:?} => {{\n{mapped}}}")
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
}

fn toggle_egui(
    mut msgs: MessageReader<KeyboardInput>,
    mut show_egui: ResMut<ShowEgui>,
    mut q_debug: Query<&mut Node, With<DebugText>>,
) {
    for msg in msgs.read() {
        if msg.key_code == KeyCode::F2 && msg.state == ButtonState::Released {
            show_egui.0 = !show_egui.0;
            for mut text in &mut q_debug {
                match text.display {
                    Display::None => text.display = Display::DEFAULT,
                    _ => text.display = Display::None,
                }
            }
            info!("toggling egui {show_egui:?}");
        }
    }
}

#[derive(Component)]
struct DebugText;

#[derive(Debug, Resource, Default, PartialEq, Eq)]
struct ShowEgui(bool);
