use itertools::Itertools;

use crate::prelude::*;

pub fn handle_messages(mut messages: MessageReader<TerminalMessage>, mut commands: Commands) {
    trace!("handle message");
    let mut to_reflow = vec![];
    for msg in messages.read() {
        debug!(?msg);
        match &msg.kind {
            TerminalMessageKind::Reflow => {
                to_reflow.push(msg.target);
            }
            TerminalMessageKind::Writeln(string) => {
                commands.run_system_cached_with(writeln, (msg.target, string.clone()));
            }
            TerminalMessageKind::Scroll(direction) => {
                commands.run_system_cached_with(scroll, (msg.target, *direction));
                to_reflow.push(msg.target);
            }
            TerminalMessageKind::JumpToBottom => {
                commands.run_system_cached_with(jump_to_bottom, msg.target);
                to_reflow.push(msg.target);
            }
        }
    }
    for target in to_reflow.into_iter().unique() {
        commands.run_system_cached_with(reflow, target);
    }
    messages.clear();
}

// TODO: This completely replaces the data every time, but
// that's not efficient. we should only replace the items that are not
// already correctly populated. -- but is that worthwhile for the small
// screen space this will take up?
fn reflow(
    target: In<Entity>,
    terminfo: Query<(
        &TerminalLines,
        Ref<TerminalRows>,
        Ref<TerminalCols>,
        Ref<TerminalScrollPos>,
    )>,
    lines_q: Query<&TerminalLine>,
    mut commands: Commands,
) {
    trace!("Reflow");
    let (line_reltarget, rows, cols, scroll_pos) = r!(terminfo.get(*target));
    let mut new_rows = vec![];
    let mut line_idx = **scroll_pos;
    let mut temp = vec![];
    while new_rows.len() < **rows {
        if let Some(line_id) = line_reltarget.get(line_idx)
            && let Ok(line) = lines_q.get(*line_id)
        {
            let mut offset = 0;
            while new_rows.len() < **rows && offset < line.len() {
                let new_row = TerminalRow::new(*line_id, offset, *target);
                let id = commands.spawn(new_row).id();
                temp.push(id);
                offset += **cols;
            }
            temp.reverse();
            new_rows.append(&mut temp);
        } else {
            let id = commands.spawn(TerminalRow::empty(*target)).id();
            new_rows.push(id);
        }
        line_idx += 1;
    }
    new_rows.reverse();
    commands
        .entity(*target)
        .replace_related::<TerminalRow>(&new_rows);
}

fn writeln(In((target, line)): In<(Entity, String)>, mut commands: Commands) {
    trace!("writeline");
    let child = commands.spawn(TerminalLine::new(line, target)).id();
    commands
        .entity(target)
        .add_one_related::<TerminalLine>(child);
}

fn scroll(
    In((target, val)): In<(Entity, isize)>,
    scroll: Query<&TerminalScrollPos>,
    mut commands: Commands,
) {
    let prev = r!(scroll.get(target));
    commands
        .entity(target)
        .insert(TerminalScrollPos(prev.saturating_add_signed(val)));
}

fn jump_to_bottom(In(target): In<Entity>, mut commands: Commands) {
    commands.entity(target).insert(TerminalScrollPos(0));
}

#[test]
fn test_reflow() {
    let mut app = App::new();
    app.add_plugins((TestRunnerPlugin::default(), CommandPromptPlugin));
    let term = app
        .world_mut()
        .spawn((Terminal, TerminalRows(10), TerminalCols(15)))
        .id();

    app.add_step(0, move |mut commands: Commands| {
        commands.write_message(TerminalMessage::writeln(
            term,
            "this line has more than 15 characters",
        ));
        commands.write_message(TerminalMessage::writeln(term, "not me tho"));
        commands.write_message(TerminalMessage::writeln(term, "nor i"));
        commands.write_message(TerminalMessage::reflow(term));
        commands.set_state(Step(1));
    });
    app.add_step(
        1,
        (move |mut commands: Commands, q_term: Query<(&TerminalLayout, &TerminalRows)>| {
            let (layout, num_rows) = q_term.get(term).unwrap();
            info!("Assert row length.");
            if layout.len() != **num_rows {
                error!(?layout, ?num_rows);
                commands.write_message(AppExit::error());
            }
            commands.set_state(Step(2));
        })
        .after(handle_messages),
    );
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
        "",
        "",
        "",
        "",
        "",
        "nor i",
        "not me tho",
        "this line has m",
        "ore than 15 cha",
        "racters",
    ];
    let e = expected.clone();
    app.add_step(
        2,
        (move |mut commands: Commands,
               q_term: Query<(&TerminalLayout, &TerminalCols)>,
               rows: Query<&TerminalRow>,
               lines: Query<&TerminalLine>| {
            let (layout, num_cols) = q_term.get(term).unwrap();
            let rows = get_rows(layout, rows, lines, **num_cols);
            info!("Assert layout.");
            if rows != e {
                error!(?rows, ?e);
                commands.write_message(AppExit::error());
            }
            commands.write_message(TerminalMessage::scroll(term, 3));
            commands.set_state(Step(3));
        })
        .after(handle_messages),
    );
    let mut e = expected.clone();
    app.add_step(
        3,
        (move |mut commands: Commands,
               q_term: Query<(&TerminalLayout, &TerminalCols)>,
               rows: Query<&TerminalRow>,
               lines: Query<&TerminalLine>| {
            {
                info!("Assert scrolled layout");
                let (layout, num_cols) = q_term.get(term).unwrap();
                let rows = get_rows(layout, rows, lines, **num_cols);
                for _ in 0..3 {
                    e.pop();
                    e.insert(0, "");
                }
                if rows != e {
                    error!(?rows, ?e);
                    commands.write_message(AppExit::error());
                }
                commands.write_message(TerminalMessage::jump_to_bottom(term));
                commands.set_state(Step(4));
            }
        })
        .after(handle_messages),
    );
    app.add_step(
        4,
        (move |mut commands: Commands,
               q_term: Query<(&TerminalLayout, &TerminalCols)>,
               rows: Query<&TerminalRow>,
               lines: Query<&TerminalLine>| {
            info!("Assert scrolled layout");
            let (layout, num_cols) = q_term.get(term).unwrap();
            let rows = get_rows(layout, rows, lines, **num_cols);
            if rows != expected {
                error!(?rows, ?expected);
                commands.write_message(AppExit::error());
            }
            commands.write_message(TerminalMessage::jump_to_bottom(term));
        })
        .after(handle_messages),
    );
    assert!(app.run().is_success())
}
