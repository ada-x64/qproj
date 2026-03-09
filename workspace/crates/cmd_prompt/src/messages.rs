use bevy::platform::collections::HashMap;

use crate::prelude::*;

pub fn handle_messages(
    mut messages: MessageReader<TermMsg>,
    mut commands: Commands,
    q_terminfo: Query<TermInfo>,
    q_lines: Query<(Entity, &TerminalLine)>,
    q_spans: Query<(Entity, &VirtualTextSpan)>,
) {
    trace!("handle window message");
    let mut to_reflow = vec![];
    let mut to_write = HashMap::<Entity, Vec<&VirtualTextSpanSpawner>>::new();
    for msg in messages.read() {
        match &msg.kind {
            TermMsgKind::Reflow => {
                if !to_reflow.contains(&msg.target) {
                    to_reflow.push(msg.target);
                }
            }
            TermMsgKind::Scroll(dir) => {
                commands.run_system_cached_with(scroll, (msg.target, *dir));
            }
            TermMsgKind::JumpToBottom => {
                commands.run_system_cached_with(jump_to_bottom, msg.target);
            }
            TermMsgKind::Write(spawners) => {
                to_write.entry(msg.target).or_default().extend(spawners);
            }
        }
    }
    for target in to_reflow.into_iter() {
        commands.run_system_cached_with(reflow, target);
    }
    for (target, vec) in to_write {
        // do NOT want to clear if we can't get terminfo.
        // try again next frame.
        let terminfo = r!(q_terminfo.get(target));
        let mut performer = AnsiPerformer::new(&terminfo);
        let mut stream = AnsiParser::new();
        for spawner in vec {
            // parse ansi
            performer.set_default_style(spawner.style);
            for byte in spawner.text.as_bytes() {
                stream.advance(&mut performer, *byte);
            }
        }
        performer.execute(&mut commands, &q_lines, &q_spans);
    }
    messages.clear();
}

/// Takes a TerminalLine and returns a vec of newly spawned TerminalRows.
fn flow_line(
    commands: &mut Commands,
    window_id: Entity,
    line_id: Entity,
    line_len: usize,
    num_cols: usize,
) -> Vec<Entity> {
    trace!("flow line");
    if num_cols == 0 {
        return vec![];
    }
    let mut res = vec![];
    let mut offset = 0;
    while offset < line_len {
        let new_row = TerminalRow::new(window_id, line_id, offset);
        let id = commands.spawn(new_row).id();
        res.push(id);
        offset += num_cols;
    }
    res
}

/// Reflows the entire underlying buffer.
fn reflow(
    id: In<Entity>,
    terminfo: Query<TermInfo>,
    q_lines: Query<(Entity, &TerminalLine)>,
    mut commands: Commands,
) {
    trace!("Reflow");
    let terminfo = r!(terminfo.get(*id));
    if **terminfo.num_cols == 0 {
        return;
    }
    commands.entity(*id).despawn_related::<TerminalLayout>();
    for (line_id, line) in terminfo.grid_lines(&q_lines).collect::<Vec<_>>() {
        flow_line(&mut commands, *id, line_id, line.len(), **terminfo.num_cols);
    }
}

fn scroll(
    In((window_id, val)): In<(Entity, isize)>,
    q: Query<(&TerminalScrollPos, &TerminalLayout, &Children)>,
    mut commands: Commands,
) {
    let (prev, rows, children) = r!(q.get(window_id));
    commands.entity(window_id).insert(TerminalScrollPos(
        prev.saturating_add_signed(val)
            .clamp(0, rows.len().saturating_sub(children.len())),
    ));
}

fn jump_to_bottom(In(target): In<Entity>, mut commands: Commands) {
    commands.entity(target).insert(TerminalScrollPos(0));
}

// #[test]
// fn test_reflow() {
//     let mut app = App::new();
//     app.add_plugins(test_harness);
//     let term = app.world_mut().spawn(Terminal).id();
//     let term_window = app
//         .world_mut()
//         .spawn((
//             Terminal,
//             Node {
//                 width: px(150),
//                 height: px(500),
//                 ..Default::default()
//             },
//             TextFont {
//                 font_size: 10.,
//                 ..Default::default()
//             },
//         ))
//         .id();

//     app.add_step(0, move |mut commands: Commands| {
//         commands.write_message(TermMsg::write(
//             term_window,
//             "this line has more than 15 characters\n",
//         ));
//         commands.write_message(TermMsg::write(term, "not me tho\n"));
//         commands.write_message(TermMsg::write(term, "nor i\n"));
//         commands.write_message(TermMsg::reflow(term_window));
//         commands.set_state(Step(1));
//     });
//     // TODO: This test will need updated when supporting rich text.
//     let get_rows = |layout: &TerminalLayout,
//                     rows: Query<&TerminalRow>,
//                     lines: Query<&TerminalLine>,
//                     num_cols: usize| {
//         layout
//             .iter()
//             .filter_map(|row_id| {
//                 let row = rows.get(row_id.entity()).ok()?;
//                 row.line
//                     .map(|id| {
//                         lines
//                             .get(id)
//                             .unwrap()
//                             .value
//                             .chars()
//                             .skip(row.offset)
//                             .take(num_cols)
//                             .collect::<String>()
//                     })
//                     .or(Some(String::new()))
//             })
//             .collect::<Vec<String>>()
//     };
//     let expected = vec![
//         "this line has m",
//         "ore than 15 cha",
//         "racters",
//         "not me tho",
//         "nor i",
//     ];
//     app.add_step(
//         1,
//         (move |mut commands: Commands,
//                q_term: Query<(&TerminalLayout, &TermWidth)>,
//                rows: Query<&TerminalRow>,
//                lines: Query<&TerminalLine>| {
//             let (layout, num_cols) = q_term.get(term).unwrap();
//             let rows = get_rows(layout, rows, lines, **num_cols);
//             info!("Assert layout.");
//             if rows != expected {
//                 error!(?rows, ?expected);
//                 commands.write_message(AppExit::error());
//             } else {
//                 commands.write_message(AppExit::Success);
//             }
//         })
//         .after(TerminalSystems::PostMutation),
//     );
//     assert!(app.run().is_success())
// }
