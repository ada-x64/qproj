use bevy::platform::collections::HashMap;

use crate::prelude::*;

fn retry_message(commands: &mut Commands, msg: TermMsg, reason: &str) {
    if msg.retry_count < 10 {
        // try again next frame
        commands.write_message(TermMsg {
            retry_count: msg.retry_count + 1,
            ..msg
        });
    } else {
        warn!(?msg, reason, "Dropped terminal message");
    }
}

pub fn handle_messages(
    mut messages: MessageReader<TermMsg>,
    mut commands: Commands,
    q_terminfo: Query<TermInfo>,
    q_lines: Query<(Entity, &VtLine, &VtRowTarget)>,
    q_rows: Query<(Entity, &VtRow, Option<&VtViewportRow>)>,
) {
    trace!("handle window message");
    let mut to_reflow = vec![];
    let mut to_write = HashMap::<Entity, Vec<&TermWrite>>::new();
    for msg in messages.read() {
        match &msg.kind {
            TermMsgKind::Reflow => {
                if !to_reflow.contains(&msg.target) {
                    to_reflow.push(msg.target);
                }
            }
            // TODO Make a system to respond to changes in VtScrollPos
            TermMsgKind::Scroll(dir) => {
                let terminfo = q_terminfo.get(msg.target);
                if terminfo.is_err() {
                    retry_message(&mut commands, msg.clone(), "no terminfo");
                    continue;
                }
                let terminfo = terminfo.unwrap();
                commands
                    .entity(terminfo.id)
                    .insert(VtScrollPos(terminfo.scroll_pos.saturating_sub_signed(*dir)));
            }
            TermMsgKind::JumpToBottom => {
                commands.entity(msg.target).insert(VtScrollPos(0));
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
        let mut grid = Grid::new(&terminfo, &q_lines, &q_rows);
        {
            let mut performer = AnsiPerformer::new(&mut grid);
            let mut stream = AnsiParser::new();
            for spawner in vec {
                // parse ansi
                if let Some(style) = spawner.style {
                    performer.reset_style(style);
                } else if spawner.reset_style {
                    performer.reset_style(VtCellStyle::default());
                }
                for byte in spawner.text.as_bytes() {
                    stream.advance(&mut performer, *byte);
                }
            }
        }
        grid.sync(&mut commands);
    }
    messages.clear();
}

/// Takes a [`VtLine`] and returns a vec of newly spawned [`VtRow`]s.
fn flow_line(
    commands: &mut Commands,
    terminfo: &TermInfoItem<'_, '_>,
    line_id: Entity,
    line: &VtLine,
) -> Vec<Entity> {
    trace!("flow line");
    let mut res = vec![];
    if terminfo.size.cols == 0 || terminfo.size.rows == 0 {
        return res;
    }
    let mut offset = 0;
    while offset < line.cells().len() {
        let new_row = VtRow::new(line_id, offset);
        let id = commands.spawn(new_row).id();
        res.push(id);
        offset += terminfo.size.cols;
    }
    res
}

/// Reflows the entire underlying buffer.
fn reflow(
    id: In<Entity>,
    terminfo: Query<TermInfo>,
    q_lines: Query<(Entity, &VtLine)>,
    q_rows: Query<Entity, (With<VtRow>, Without<VtViewportRow>)>,
    mut commands: Commands,
) {
    trace!("Reflow");
    let terminfo = r!(terminfo.get(*id));
    // clear terminal display cache
    q_rows.iter().for_each(|row_id| {
        commands.entity(row_id).despawn();
    });
    commands.entity(*id).despawn_related::<VtViewport>();
    // exit early if nothing to do
    if terminfo.size.cols == 0 || terminfo.size.rows == 0 {
        return;
    }
    // reflow
    let rows = terminfo
        .lines(&q_lines)
        .fold(vec![], |mut res, (line_id, line)| {
            let mut rows = flow_line(&mut commands, &terminfo, line_id, line);
            res.append(&mut rows);
            res
        });
    trace!("spawned rows");
    // spawn
    let row_ids = rows
        .into_iter()
        .rev()
        .skip(terminfo.scroll_pos.0)
        .take(terminfo.size.rows)
        .collect::<Vec<_>>();
    for id in row_ids.into_iter().rev() {
        commands.entity(id).insert(VtViewportRow::new(terminfo.id));
    }
    trace!("spawned viewport rows");
}

// fn scroll(
//     In((window_id, val)): In<(Entity, isize)>,
//     q: Query<(&VtScrollPos, &VtLayout, &Children)>,
//     mut commands: Commands,
// ) {
//     let (prev, rows, children) = r!(q.get(window_id));
//     commands.entity(window_id).insert(VtScrollPos(
//         prev.saturating_add_signed(val)
//             .clamp(0, rows.len().saturating_sub(children.len())),
//     ));
// }

// fn jump_to_bottom(In(target): In<Entity>, mut commands: Commands) {
//     commands.entity(target).insert(VtScrollPos(0));
// }

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
