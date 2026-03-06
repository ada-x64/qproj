use anstyle_parse::Utf8Parser;

use crate::prelude::*;

enum AnsiAction {
    Write {
        line: usize,
        offset: usize,
        text: char,
    },
    SetCursorPos {
        line: usize,
        char: usize,
    },
    Erase {
        // TODO - Can only support this partially,
        // since we're not directly modifying the displayed grid
    },
}

/// Select C0/C1 control codes. See
/// https://en.wikipedia.org/wiki/C0_and_C1_control_codes
#[repr(u8)]
enum ControlCodes {
    /// Bell, ^G, \a
    BEL = 0x07,
    /// Backspace, ^H, \b
    BS = 0x08,
    /// Tab, ^I, \t
    HT = 0x09,
    /// Line feed, ^J, \n
    LF = 0x0a,
    /// Carriage return, ^M, \r
    CR = 0x0d,
    /// Escape, ^[, \x1b, \033
    ESC = 0x1b,
}

#[repr(u8)]
enum CsiAction {
    // cursor actions
    /// Cursor up, n rows
    CUU = 0x41,
    /// Cursor down, n rows
    CUD = 0x42,
    /// Cursor forward, n cols
    CUF = 0x43,
    /// Cursor back, n cols
    CUB = 0x44,
    /// Cursor to start of line, n lines down
    CNL = 0x45,
    /// Cursor to start of line, n lines up
    CPL = 0x46,
    /// Cursor to column n
    CHA = 0x47,
    /// Cursor to row n, col m
    CUP = 0x48,
    /// Same as CUP
    HVP = 0x66,
    // erase
    /// Erase parts of the screen.
    ED = 0x4a,
    /// Erase around cursor.
    EL = 0x4b,

    // color
    /// SGR style escapes
    SGR = 0x6d,
}

struct NewItem<T: Component> {
    item: T,
    line_idx: usize,
    replacement_target: Option<Entity>,
}

/// Parses the passed [`VirtualTextSpanSpawner`], possibly expanding it into multiple
/// and modifying various [`TerminalLine`]s.
struct AnsiPerformer<'a> {
    lines: &'a TerminalLines, // note: VirtualTextSpans are children of TerminalLine entities
    cursor: &'a mut TerminalCursor,
    new_lines: Vec<NewItem<TerminalLine>>,
    new_vspans: Vec<NewItem<VirtualTextSpan>>,
    actions: Vec<AnsiAction>,
    // todo? default style info from VirtualTextSpanSpawner
}
impl<'a> AnsiPerformer<'a> {
    fn new(lines: &'a TerminalLines, cursor: &'a mut TerminalCursor) -> Self {
        Self {
            lines,
            cursor,
            actions: vec![],
            new_lines: vec![],
            new_vspans: vec![],
        }
    }
    /// Returns vectors of new components to add or replace. If the optional
    /// entity is Some, then the result is to be replaced.
    fn execute(self, commands: &mut Commands) {
        // TODO: Iterate through self.actions and perform commands to spawn
        // and modify entities as appropriate
    }
}
impl<'a> anstyle_parse::Perform for AnsiPerformer<'a> {
    fn print(&mut self, c: char) {
        self.actions.push(AnsiAction::Write {
            line: self.cursor.line,
            offset: self.cursor.char,
            text: c,
        });
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            byte if byte == ControlCodes::BEL as u8 => {
                // TODO: sound a bell :)
            }
            byte if byte == ControlCodes::LF as u8 => self.actions.push(AnsiAction::SetCursorPos {
                line: self.cursor.line + 1,
                char: 0,
            }),
            byte if byte == ControlCodes::CR as u8 => self.actions.push(AnsiAction::SetCursorPos {
                line: self.cursor.line,
                char: 0,
            }),
            _ => {
                info_once!(
                    "Given an unsupported escape C0 or C1 escape sequence. See the docs supported types."
                );
                // unsupported
            }
        }
    }

    // TODO: figure out what the hell all this means
    // look at example impls
    fn hook(
        &mut self,
        _params: &anstyle_parse::Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: u8,
    ) {
    }

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        params: &anstyle_parse::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: u8,
    ) {
        let iter = params.iter();
        match action {
            action if action == CsiAction::CUU as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    // TODO: what's being returned here?
                    line: self.cursor.line + (iter.next().unwrap_or(0) as usize),
                    char: self.cursor.char,
                });
            }
            action if action == CsiAction::CUD as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line + iter.next().unwrap_or(0) as usize,
                    char: self.cursor.char,
                });
            }
            action if action == CsiAction::CUF as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line,
                    char: self.cursor.char + iter.next().unwrap_or(0) as usize,
                })
            }
            action if action == CsiAction::CUB as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line,
                    char: self.cursor.char - iter.next().unwrap_or(0) as usize,
                });
            }
            action if action == CsiAction::CNL as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line + iter.next().unwrap_or(0) as usize,
                    char: 0,
                });
            }
            action if action == CsiAction::CPL as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line - iter.next().unwrap_or(0) as usize,
                    char: 0,
                });
            }
            action if action == CsiAction::CHA as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line,
                    char: iter.next().unwrap_or(0) as usize,
                });
            }
            action if action == CsiAction::CUP as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: iter.next().unwrap_or(0) as usize,
                    char: iter.next().unwrap_or(0) as usize,
                });
            }
            action if action == CsiAction::HVP as u8 => {
                self.actions.push(AnsiAction::SetCursorPos {
                    line: iter.next().unwrap_or(0) as usize,
                    char: iter.next().unwrap_or(0) as usize,
                });
            }
            // action if action == CsiAction::ED as u8 => {
            //     self.actions.push(AnsiAction::Erase { mode: iter.next().unwrap_or(0) as usize });
            // }
            // action if action == CsiAction::EL as u8 => {
            //     self.actions.push(AnsiAction::Erase { mode: iter.next().unwrap_or(0) as usize });
            // }
            action if action == CsiAction::SGR as u8 => {
                // TODO: Style stuff with anstyle
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

pub fn handle_terminal_messages(
    mut messages: MessageReader<TerminalMessage>,
    mut commands: Commands,
    q_windows: Query<(&WindowList, &TerminalLines, &mut TerminalCursor), With<Terminal>>,
    q_cols: Query<&TermWidth, With<TerminalWindow>>,
) {
    trace!("handle terminal message");
    for msg in messages.read() {
        match &msg.kind {
            TerminalMessageKind::Writeln(spawners) => {
                // do NOT want to clear if we can't get the window.
                // try again next frame.
                let (windows, lines, mut cursor) = r!(q_windows.get_mut(msg.target));
                let (text, children) =
                    spawners
                        .iter()
                        .fold((String::new(), vec![]), |(text, ids), spawner| {
                            // parse ansi
                            let mut stream = anstyle_parse::Parser::<Utf8Parser>::new();
                            let mut performer = AnsiPerformer::new(lines, &mut cursor);
                            for byte in spawner.text.as_bytes() {
                                stream.advance(&mut performer, *byte);
                            }

                            // add span
                            let range = spawner.text.len();
                            let child = commands
                                .spawn(VirtualTextSpan {
                                    range,
                                    color: spawner.color,
                                    background_color: spawner.background_color,
                                })
                                .id();
                            (
                                text + &spawner.text,
                                [ids, vec![child]].into_iter().flatten().collect::<Vec<_>>(),
                            )
                        });
                let len = text.len();
                let line_id = commands
                    .spawn(TerminalLine::new(msg.target, text))
                    .add_children(&children)
                    .id();
                commands
                    .entity(msg.target)
                    .add_one_related::<TerminalLine>(line_id);
                for window_id in windows.iter() {
                    let num_cols = c!(q_cols.get(window_id));
                    flow_line(&mut commands, window_id, line_id, len, **num_cols);
                }
            }
        }
    }
    messages.clear();
}

pub fn handle_window_messages(
    mut messages: MessageReader<TerminalWindowMessage>,
    mut commands: Commands,
    q_window: Query<&TerminalWindow>,
) {
    trace!("handle window message");
    let mut to_reflow = vec![];
    for msg in messages.read() {
        match &msg.kind {
            TerminalWindowMsgKind::Reflow => {
                if !to_reflow.contains(&msg.target) {
                    to_reflow.push(msg.target);
                }
            }
            TerminalWindowMsgKind::Scroll(dir) => {
                commands.run_system_cached_with(scroll, (msg.target, *dir));
            }
            TerminalWindowMsgKind::JumpToBottom => {
                commands.run_system_cached_with(jump_to_bottom, msg.target);
            }
            TerminalWindowMsgKind::Terminal(tmsg) => {
                warn_once!(
                    "Using TerminalWindowMsgKind::Terminal results in a frame delay and extra memory allocations to clone the message data. Try directly sending a TerminalWindowMessage instead."
                );
                let window = c!(q_window.get(msg.target));
                commands.write_message(TerminalMessage::new(window.0, (*tmsg).clone()));
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
    window_id: In<Entity>,
    term_width: Query<&TermWidth, With<TerminalWindow>>,
    terminfo: Query<(&WindowList, &TerminalLines), With<Terminal>>,
    lines_q: Query<&TerminalLine>,
    mut commands: Commands,
) {
    trace!("Reflow");
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
        flow_line(&mut commands, *window_id, line_id, line.len(), **cols);
    })
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
        commands.write_message(TerminalMessage::writeln(term, "not me tho"));
        commands.write_message(TerminalMessage::writeln(term, "nor i"));
        commands.write_message(TerminalWindowMessage::reflow(term_window));
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
