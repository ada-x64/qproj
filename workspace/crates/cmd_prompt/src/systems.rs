use crate::prelude::*;

// How to do this?
// TextPipeline measures the text nodes automatically and does not expose
// a way to directly measure a string of text.
// Whenever the text font changes, we should get the computed text block for
// a display:hidden text node which contains only a single " " character.
// This way we avoid directly measuring the font.
// pub fn measure_cwidth(
//     q: Query<&TextFont, (With<TerminalWindow>, Changed<TextFont>)>,
//     mut commands: Commands,
//     fonts: Res<Assets<Font>>,
// ) {
//     todo!()
// }

// TODO: This messes up in headless tests because the node size is always set to 0x0
pub fn resize(
    q: Query<(&TerminalWindow, &ComputedNode, &TextFont), Changed<ComputedNode>>,
    mut commands: Commands,
) {
    trace!("resize");
    for (window, node, font) in q.iter() {
        // TODO: Calculate monospace character width
        let character_width = font.font_size;
        let size = node.size();
        let width = (size.x / character_width).floor() as usize;
        let height = (size.y / font.font_size).floor() as usize;
        commands.entity(window.0).insert(TermWidth(width));
        commands.entity(window.0).insert(TermHeight(height));
        debug!("Got node size: {size:?}");
        debug!("Set new term size to {width} x {height}");
    }
}

pub fn update_layout(
    q: Query<
        (
            &TermHeight,
            &TermWidth,
            &TerminalLayout,
            &TerminalWindowList,
        ),
        (
            Or<(
                Changed<TermHeight>,
                Changed<TermWidth>,
                Changed<TerminalLayout>,
                Changed<TerminalLines>,
            )>,
            With<Terminal>,
        ),
    >,
    q_lines: Query<&TerminalLine>,
    q_rows: Query<&TerminalRow>,
    q_windows: Query<&TerminalScrollPos, With<TerminalWindow>>,
    mut commands: Commands,
) {
    trace!("update layout");
    for (num_rows, num_cols, layout, window_list) in q {
        window_list.iter().for_each(|window| {
            let scroll_pos = r!(q_windows.get(window));
            let new_children = layout
                .iter()
                .rev()
                .skip(**scroll_pos)
                .take(**num_rows)
                .filter_map(|id| {
                    let row = r!(q_rows.get(id));
                    let line = r!(q_lines.get(r!(row.line)));
                    let textval = line
                        .chars()
                        .skip(row.offset)
                        .take(**num_cols)
                        .collect::<String>();
                    Some(commands.spawn(TextSpan::new(textval + "\n")).id())
                })
                .rev()
                .collect::<Vec<_>>();
            commands.entity(window).despawn_children();
            commands.entity(window).add_children(&new_children);
        });
    }
}

#[test]
fn test_text_nodes() {
    let mut app = App::new();
    app.add_plugins(test_harness);
    app.add_step(0, |mut commands: Commands| {
        let term_id = commands.spawn(Terminal).id();
        commands.spawn((
            TextFont {
                font_size: 10.,
                ..Default::default()
            },
            Node {
                width: px(100),
                height: px(50),
                ..Default::default()
            },
            TerminalWindow(term_id),
            TerminalScrollPos(10),
        ));
        for i in 0..100 {
            commands.write_message(TerminalMessage::writeln(term_id, i.to_string()));
        }
        commands.set_state(Step(1));
    });
    app.add_step(
        1,
        (|mut commands: Commands,
          window: Query<(&Children, &TerminalWindow, &TerminalScrollPos)>,
          terminal: Query<(&TerminalLayout, &TermHeight), With<Terminal>>,
          text_spans: Query<&TextSpan>,
          rows: Query<&TerminalRow>,
          lines: Query<&TerminalLine>| {
            let window = window.single();
            if window.is_err() {
                error!("TerminalWindow with Children not found!");
                commands.write_message(AppExit::error());
                return;
            }
            let (children, window, scroll_pos) = window.unwrap();
            let spans = children
                .iter()
                .filter_map(|child_id| Some(r!(text_spans.get(child_id)).0.clone()))
                .collect::<Vec<_>>();
            let (layout, num_rows) = r!(terminal.get(window.0));
            let mut lines = layout
                .iter()
                .filter_map(|row_id| {
                    let row = r!(rows.get(row_id));
                    let line_id = r!(row.line);
                    let line = r!(lines.get(line_id));
                    Some(line.value.clone())
                })
                .rev()
                .skip(**scroll_pos)
                .take(**num_rows)
                .collect::<Vec<_>>();
            lines.reverse();
            let expected = (85..=89).map(|i| i.to_string()).collect::<Vec<String>>();
            info!(?spans, ?lines, ?expected);
            if spans == lines && spans == expected {
                commands.write_message(AppExit::Success);
            } else {
                commands.write_message(AppExit::error());
            }
        })
        .after(TerminalSystems),
    );
    assert!(app.run().is_success());
}
