use crate::prelude::*;

pub fn handle_messages(mut messages: MessageReader<TerminalMessage>, mut commands: Commands) {
    trace!("handle message");
    let mut to_reflow = vec![];
    for msg in messages.read() {
        match &msg.kind {
            TerminalMessageKind::Reflow => {
                if !to_reflow.contains(&msg.window_id) {
                    to_reflow.push(msg.window_id);
                }
            }
            TerminalMessageKind::Writeln(string) => {
                commands.run_system_cached_with(writeln, (msg.window_id, string.clone()));
            }
            TerminalMessageKind::Scroll(direction) => {
                commands.run_system_cached_with(scroll, (msg.window_id, *direction));
            }
            TerminalMessageKind::JumpToBottom => {
                commands.run_system_cached_with(jump_to_bottom, msg.window_id);
            }
        }
    }
    for target in to_reflow.into_iter() {
        commands.run_system_cached_with(reflow, target);
    }
    messages.clear();
}

/// Takes a TerminalLine and returns a vec of newly spawned TerminalRows.
fn flow_line(
    commands: &mut Commands,
    line_id: Entity,
    text: &str,
    term_id: Entity,
    num_cols: usize,
) -> Vec<Entity> {
    trace!("flow line");
    if num_cols == 0 {
        return vec![];
    }
    let mut res = vec![];
    let mut offset = 0;
    while offset < text.len() {
        let new_row = TerminalRow::new(term_id, line_id, offset);
        let id = commands.spawn(new_row).id();
        res.push(id);
        offset += num_cols;
    }
    res
}

/// Reflows the entire underlying buffer.
fn reflow(
    window_id: In<Entity>,
    term_width: Query<&TermWidth, With<TerminalWindow>>,
    terminfo: Query<(&TerminalWindowList, &TerminalLines), With<Terminal>>,
    lines_q: Query<&TerminalLine>,
    mut commands: Commands,
) {
    trace!("Reflow");
    debug!(?window_id);
    let cols = r!(term_width.get(*window_id));
    let lines_reltarget = terminfo
        .iter()
        .find_map(|(list, lines)| list.contains(&window_id).then_some(lines));
    let lines_reltarget = r!(lines_reltarget);
    if **cols == 0 {
        return;
    }
    commands
        .entity(*window_id)
        .despawn_related::<TerminalLayout>();
    // todo: batch process this in parallel
    lines_reltarget.iter().for_each(|line_id| {
        let line = r!(lines_q.get(line_id));
        flow_line(&mut commands, line_id, line, *window_id, **cols);
    })
}

fn writeln(
    In((window_id, text)): In<(Entity, String)>,
    mut commands: Commands,
    q_cols: Query<&TermWidth>,
    q_terms: Query<(Entity, &TerminalWindowList)>,
) {
    trace!("writeline");
    debug!(?window_id);
    let num_cols = r!(q_cols.get(window_id));
    let term_id = q_terms
        .iter()
        .find_map(|(entity, list)| list.contains(&window_id).then_some(entity));
    let term_id = r!(term_id);
    let new_line_id = commands
        .spawn(TerminalLine::new(term_id, text.clone()))
        .id();
    flow_line(&mut commands, new_line_id, &text, window_id, **num_cols);
}

fn scroll(
    In((window_id, val)): In<(Entity, isize)>,
    q: Query<(&TerminalScrollPos, &TerminalLayout, &Children)>,
    mut commands: Commands,
) {
    let (prev, rows, children) = r!(q.get(window_id));
    commands.entity(window_id).insert(TerminalScrollPos(
        prev.saturating_add_signed(val)
            .clamp(0, rows.len() - children.len()),
    ));
}

fn jump_to_bottom(In(target): In<Entity>, mut commands: Commands) {
    commands.entity(target).insert(TerminalScrollPos(0));
}

#[test]
fn test_reflow() {
    let mut app = App::new();
    app.add_plugins(test_harness);
    let term = app.world_mut().spawn(Terminal).id();
    let term_window = app
        .world_mut()
        .spawn((
            TerminalWindow(term),
            Node {
                width: px(150),
                height: px(500),
                ..Default::default()
            },
            TextFont {
                font_size: 10.,
                ..Default::default()
            },
        ))
        .id();

    app.add_step(0, move |mut commands: Commands| {
        commands.write_message(TerminalMessage::writeln(
            term_window,
            "this line has more than 15 characters",
        ));
        commands.write_message(TerminalMessage::writeln(term_window, "not me tho"));
        commands.write_message(TerminalMessage::writeln(term_window, "nor i"));
        commands.write_message(TerminalMessage::reflow(term_window));
        commands.set_state(Step(1));
    });
    // TODO: This test will need updated when supporting rich text.
    let get_rows = |layout: &TerminalLayout,
                    rows: Query<&TerminalRow>,
                    lines: Query<&TerminalLine>,
                    num_cols: usize| {
        layout
            .iter()
            .filter_map(|row_id| {
                let row = rows.get(row_id.entity()).ok()?;
                row.line
                    .map(|id| {
                        lines
                            .get(id)
                            .unwrap()
                            .value
                            .chars()
                            .skip(row.offset)
                            .take(num_cols)
                            .collect::<String>()
                    })
                    .or(Some(String::new()))
            })
            .collect::<Vec<String>>()
    };
    let expected = vec![
        "this line has m",
        "ore than 15 cha",
        "racters",
        "not me tho",
        "nor i",
    ];
    app.add_step(
        1,
        (move |mut commands: Commands,
               q_term: Query<(&TerminalLayout, &TermWidth)>,
               rows: Query<&TerminalRow>,
               lines: Query<&TerminalLine>| {
            let (layout, num_cols) = q_term.get(term).unwrap();
            let rows = get_rows(layout, rows, lines, **num_cols);
            info!("Assert layout.");
            if rows != expected {
                error!(?rows, ?expected);
                commands.write_message(AppExit::error());
            } else {
                commands.write_message(AppExit::Success);
            }
        })
        .after(TerminalSystems::PostMutation),
    );
    assert!(app.run().is_success())
}
