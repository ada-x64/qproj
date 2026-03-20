use crate::prelude::*;

/// Bevy logo in ANSI art using cursor-movement escape sequences.
///
/// Tests CUP (cursor absolute position), truecolor foreground, and overwrite
/// behavior when segments overlap. Based on a Python reference script that
/// renders three stylized bird/swoosh shapes using box-drawing characters.
///
/// Escape sequences exercised:
///   ESC[{row};{col}H  — CUP, cursor absolute position (1-indexed)
///   ESC[38;2;R;G;Bm   — 24-bit truecolor foreground
///   ESC[1m             — bold (not yet supported — should be a no-op)
///   ESC[0m             — SGR reset
///
/// Verifies:
///   - CUP positions characters at the correct (row, col) in the grid
///   - Overlapping segments overwrite earlier content (not insert)
///   - Truecolor foreground is applied correctly
///   - Final cursor position after trailing CUP
#[test]
fn bevy_logo() {
    let mut app = get_test_app();

    // 256-color grayscale → truecolor equivalents
    // gray level = 8 + (index - 232) * 10
    // BODY(235) → 38  SHADE(238) → 68
    let body = (38u8, 38u8, 38u8);
    let shade = (68u8, 68u8, 68u8);

    fn cup(row: usize, col: usize) -> String {
        format!("\x1b[{row};{col}H")
    }
    fn fg(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{r};{g};{b}m")
    }

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

    let mut out = String::from("\x1b[1m"); // bold (no-op, should not crash)
    for (bird, br, bc) in placements {
        for &(dr, dc, text, (r, g, b)) in *bird {
            out.push_str(&cup(CANVAS_ROW + br + dr, CANVAS_COL + bc + dc));
            out.push_str(&fg(r, g, b));
            out.push_str(text);
        }
    }
    out.push_str(&cup(CANVAS_ROW + 16, 1)); // cursor below art
    out.push_str("\x1b[0m"); // reset

    let input = out;

    app.add_systems(Startup, move |mut commands: Commands| {
        let term_id = commands.spawn(Terminal).id();
        commands
            .entity(term_id)
            .insert(VtSize { cols: 40, rows: 24 });
        commands.write_message(TermMsg::write(term_id, &input));
    });

    let body_style = VtCellStyle {
        color: Color::srgb_u8(38, 38, 38),
        background: Color::BLACK,
    };
    let shade_style = VtCellStyle {
        color: Color::srgb_u8(68, 68, 68),
        background: Color::BLACK,
    };

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

            let cell_at = |row: usize, col: usize| -> Option<VtCell> {
                lines.get(row).and_then(|(_, line)| line.cells().get(col).copied())
            };

            // Enough lines for the full logo (rows 0–13)
            assert!(
                lines.len() >= 14,
                "Expected at least 14 lines, got {}",
                lines.len()
            );

            // ── bird 3 (top-right) ──
            // CUP(2,20) → row 1, col 19: first cell of bird3
            let c = cell_at(1, 19).expect("bird3 top-left");
            assert_eq!(c.value, '▄');
            assert_eq!(c.style, body_style, "bird3 top-left color");

            // ── bird 2 (middle) ──
            // CUP(5,12) → row 4, col 11: first cell of bird2
            let c = cell_at(4, 11).expect("bird2 top-left");
            assert_eq!(c.value, '▄');
            assert_eq!(c.style, body_style, "bird2 top-left color");

            // ── bird 1 (bottom-left) ──
            // CUP(8,4) → row 7, col 3: first cell of bird1
            let c = cell_at(7, 3).expect("bird1 top-left");
            assert_eq!(c.value, '▄');
            assert_eq!(c.style, body_style, "bird1 top-left color");

            // ── overwrite checks (shade segments overwrite body) ──

            // bird 1: segment (3,5,"▀▀",SHADE) overwrites body at row 10, cols 8–9
            let c = cell_at(10, 8).expect("bird1 shade overwrite");
            assert_eq!(c.value, '▀');
            assert_eq!(c.style, shade_style, "bird1 shade color");

            // bird 2: segment (3,6,"▀",SHADE) overwrites body at row 7, col 17
            let c = cell_at(7, 17).expect("bird2 shade overwrite");
            assert_eq!(c.value, '▀');
            assert_eq!(c.style, shade_style, "bird2 shade color");

            // bird 3: segment (2,4,"▄",SHADE) overwrites body at row 3, col 23
            let c = cell_at(3, 23).expect("bird3 shade overwrite");
            assert_eq!(c.value, '▄');
            assert_eq!(c.style, shade_style, "bird3 shade color");

            // ── verify body wasn't corrupted next to shade overwrites ──

            // bird 1 row 10: body cell at col 7 (left of shade)
            let c = cell_at(10, 7).expect("bird1 body adjacent to shade");
            assert_eq!(c.value, '█');
            assert_eq!(c.style, body_style, "bird1 body adjacent");

            // bird 1 row 10: body cell at col 10 (right of shade)
            let c = cell_at(10, 10).expect("bird1 body right of shade");
            assert_eq!(c.value, '█');
            assert_eq!(c.style, body_style, "bird1 body right of shade");

            // ── final cursor: CUP(18,1) → row 17, col 0 ──
            assert_eq!(terminfo.cursor.row, 17, "final cursor row");
            assert_eq!(terminfo.cursor.col, 0, "final cursor col");

            commands.write_message(AppExit::Success);
        },
    );

    assert!(app.run().is_success());
}
