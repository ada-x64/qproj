use bevy::color::palettes::{basic, css};
use crate::prelude::*;

/// Verifies that 24-bit ANSI color escapes (SGR 38;2;r;g;b for foreground,
/// 48;2;r;g;b for background) produce VtCells with the correct styles.
///
/// Input: "\x1b[38;2;255;0;128;48;2;0;64;255mHi\x1b[0m!"
///   - "Hi" should be fg=(255,0,128) bg=(0,64,255)
///   - "!" should be default style (reset via SGR 0)
#[test]
fn truecolor() {
    let mut app = get_test_app();

    app.add_systems(Startup, |mut commands: Commands| {
        let term_id = commands.spawn(Terminal).id();
        commands
            .entity(term_id)
            .insert(VtSize { cols: 80, rows: 24 });
        commands.write_message(TermMsg::write(
            term_id,
            "\x1b[38;2;255;0;128;48;2;0;64;255mHi\x1b[0m!",
        ));
    });

    app.add_step(
        0,
        |q_term: Query<TermInfo>,
         q_lines: Query<(Entity, &VtLine)>,
         mut commands: Commands| {
            let terminfo = r!(q_term.single());
            let lines: Vec<_> = terminfo.lines(&q_lines).collect();
            if lines.is_empty() {
                return;
            }

            let (_, line) = &lines[0];
            assert_eq!(line.as_string(), "Hi!");

            let cells = line.cells();
            assert_eq!(cells.len(), 3);

            let colored_style = VtCellStyle {
                color: Color::srgb_u8(255, 0, 128),
                background: Color::srgb_u8(0, 64, 255),
            };
            // 'H'
            assert_eq!(cells[0].value, 'H');
            assert_eq!(cells[0].style, colored_style);
            // 'i'
            assert_eq!(cells[1].value, 'i');
            assert_eq!(cells[1].style, colored_style);
            // '!' — after SGR reset
            assert_eq!(cells[2].value, '!');
            assert_eq!(cells[2].style, VtCellStyle::default());

            commands.write_message(AppExit::Success);
        },
    );

    assert!(app.run().is_success());
}

/// Verifies standard (30–37) and bright (90–97) ANSI palette foreground
/// colors, plus SGR 0 reset.
///
/// Tests all 8 standard + 8 bright foreground colors in a single line.
#[test]
fn palette_foreground() {
    let mut app = get_test_app();

    // Standard ANSI palette: (standard, bright)
    let expected: [(Srgba, Srgba); 8] = [
        (basic::BLACK, basic::GRAY),      // 0: black
        (css::DARK_RED, basic::RED),      // 1: red
        (basic::GREEN, basic::LIME),      // 2: green
        (basic::OLIVE, basic::YELLOW),    // 3: yellow
        (basic::NAVY, basic::BLUE),       // 4: blue
        (basic::PURPLE, basic::FUCHSIA),  // 5: magenta
        (basic::TEAL, css::AQUA),         // 6: cyan
        (basic::SILVER, basic::WHITE),    // 7: white
    ];

    // Build input: \x1b[30mA\x1b[31mB...\x1b[37mH\x1b[90mI...\x1b[97mP\x1b[0mQ
    let mut input = String::new();
    for i in 0..8u8 {
        input.push_str(&format!("\x1b[{}m{}", 30 + i, (b'A' + i) as char));
    }
    for i in 0..8u8 {
        input.push_str(&format!("\x1b[{}m{}", 90 + i, (b'I' + i) as char));
    }
    input.push_str("\x1b[0m!");

    app.add_systems(Startup, move |mut commands: Commands| {
        let term_id = commands.spawn(Terminal).id();
        commands
            .entity(term_id)
            .insert(VtSize { cols: 80, rows: 24 });
        commands.write_message(TermMsg::write(term_id, &input));
    });

    app.add_step(
        0,
        move |q_term: Query<TermInfo>,
              q_lines: Query<(Entity, &VtLine)>,
              mut commands: Commands| {
            let terminfo = r!(q_term.single());
            let lines: Vec<_> = terminfo.lines(&q_lines).collect();
            if lines.is_empty() {
                return;
            }

            let (_, line) = &lines[0];
            let cells = line.cells();
            // 8 standard + 8 bright + 1 reset char = 17
            assert_eq!(cells.len(), 17, "Expected 17 cells, got {}", cells.len());

            // Check standard colors (30–37)
            for i in 0..8usize {
                let cell = &cells[i];
                let (expected_dark, _) = expected[i];
                assert_eq!(
                    cell.style.color,
                    Color::from(expected_dark),
                    "Standard fg color {} (ESC[{}m) mismatch",
                    i,
                    30 + i
                );
                assert_eq!(cell.style.background, VtCellStyle::default().background);
            }

            // Check bright colors (90–97)
            for i in 0..8usize {
                let cell = &cells[8 + i];
                let (_, expected_bright) = expected[i];
                assert_eq!(
                    cell.style.color,
                    Color::from(expected_bright),
                    "Bright fg color {} (ESC[{}m) mismatch",
                    i,
                    90 + i
                );
            }

            // '!' — after reset
            assert_eq!(cells[16].value, '!');
            assert_eq!(cells[16].style, VtCellStyle::default());

            commands.write_message(AppExit::Success);
        },
    );

    assert!(app.run().is_success());
}
