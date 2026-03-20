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
