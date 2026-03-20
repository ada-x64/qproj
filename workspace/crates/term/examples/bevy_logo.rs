use bevy::{prelude::*, window::WindowResolution};
use q_term::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "q_term — Bevy logo (ANSI cursor art)".into(),
                resizable: true,
                resolution: WindowResolution::new(560, 420),
                ..Default::default()
            }),
            ..Default::default()
        }),
        TerminalPlugin,
    ));
    app.add_systems(Startup, setup);
    app.run();
}

fn cup(row: usize, col: usize) -> String {
    format!("\x1b[{row};{col}H")
}

fn fg(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let term_id = commands.spawn(Terminal).id();
    commands.spawn((
        Node {
            width: vw(100),
            height: vh(100),
            ..Default::default()
        },
        BackgroundColor(Color::BLACK),
        VtUi::new(term_id),
    ));

    // 256-color grayscale → truecolor: gray = 8 + (index - 232) * 10
    let body: (u8, u8, u8) = (38, 38, 38);   // index 235
    let shade: (u8, u8, u8) = (68, 68, 68);  // index 238

    type Seg = (usize, usize, &'static str, (u8, u8, u8));

    let bird1: &[Seg] = &[
        (0, 0, "▄▄▄", body),
        (1, 0, "████▄", body),
        (2, 0, "██████▄", body),
        (3, 1, "███████▄", body),
        (3, 5, "▀▀", shade),
        (4, 2, "███████", body),
        (4, 9, "▄▄", shade),
        (5, 3, "▀█████▀", body),
        (6, 5, "▀▀▀", shade),
    ];
    let bird2: &[Seg] = &[
        (0, 0, "▄▄▄", body),
        (1, 0, "█████", body),
        (2, 0, "███████", body),
        (2, 5, "▄", shade),
        (3, 1, "███████", body),
        (3, 6, "▀", shade),
        (4, 2, "▀█████", body),
        (5, 4, "▀▀▀", shade),
    ];
    let bird3: &[Seg] = &[
        (0, 0, "▄▄", body),
        (1, 0, "████", body),
        (2, 0, "█████", body),
        (2, 4, "▄", shade),
        (3, 1, "█████", body),
        (4, 2, "▀███", body),
        (5, 3, "▀▀", shade),
    ];

    const CANVAS_ROW: usize = 2;
    const CANVAS_COL: usize = 3;
    let placements: &[(&[Seg], usize, usize)] = &[
        (bird1, 6, 1),  // bottom-left
        (bird2, 3, 9),  // middle
        (bird3, 0, 17), // top-right
    ];

    let mut out = String::from("\x1b[1m");
    for (bird, br, bc) in placements {
        for &(dr, dc, text, (r, g, b)) in *bird {
            out.push_str(&cup(CANVAS_ROW + br + dr, CANVAS_COL + bc + dc));
            out.push_str(&fg(r, g, b));
            out.push_str(text);
        }
    }
    out.push_str(&cup(CANVAS_ROW + 16, 1));
    out.push_str("\x1b[0m");

    commands.write_message(TermMsg::write(term_id, out));
}
