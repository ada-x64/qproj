#![allow(clippy::upper_case_acronyms)]

use crate::prelude::*;
use anstyle_parse::{Parser, Utf8Parser};
use bevy::color::palettes::css;
use smol_str::{SmolStr, ToSmolStr};

pub type AnsiParser = Parser<Utf8Parser>;

/// Select C0/C1 control codes. See
/// https://en.wikipedia.org/wiki/C0_and_C1_control_codes
#[allow(dead_code)]
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
#[allow(dead_code)]
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

#[derive(Debug, Clone)]
enum MaybeRef<'a, T: Clone> {
    Owned(Option<Entity>, T),
    Borrowed(Entity, &'a T),
}
impl<'a, T: Clone> MaybeRef<'a, T> {
    fn value(&self) -> &T {
        match self {
            MaybeRef::Owned(_, t) => t,
            MaybeRef::Borrowed(_, t) => t,
        }
    }
    fn entity(&self) -> Option<Entity> {
        match self {
            MaybeRef::Owned(Some(e), _) => Some(*e),
            MaybeRef::Owned(None, _) => None,
            MaybeRef::Borrowed(e, _) => Some(*e),
        }
    }
}

struct WriteParams {
    text: SmolStr,
    cursor: TerminalCursor,
    style: VspanStyle,
}

type Grid<'a> = Vec<(
    MaybeRef<'a, TerminalLine>,
    Vec<MaybeRef<'a, VirtualTextSpan>>,
)>;

/// Parses the passed [`VirtualTextSpanSpawner`], possibly expanding it into multiple
/// and modifying various [`TerminalLine`]s.
pub(crate) struct AnsiPerformer<'w, 's, 'a> {
    terminfo: &'a TermInfoItem<'w, 's>,
    cursor: TerminalCursor,
    style: VspanStyle,
    default_style: VspanStyle,
    to_write: Vec<WriteParams>,
}
impl<'w, 's, 'a> AnsiPerformer<'w, 's, 'a> {
    pub fn new(terminfo: &'a TermInfoItem<'w, 's>) -> Self {
        Self {
            terminfo,
            cursor: TerminalCursor {
                char: terminfo.cursor.char,
                // TODO: This depends on the VT100 DEC mode https://vt100.net/docs/vt100-ug/chapter3.html#DECOM
                line: terminfo.cursor.line + **terminfo.scroll_pos,
            },
            style: VspanStyle::default(),
            default_style: VspanStyle::default(),
            to_write: vec![],
        }
    }
    pub fn set_default_style(&mut self, style: VspanStyle) {
        self.default_style = style;
    }

    // TODO: This logic seems bugged.
    // Getting far too many lines.
    fn write(
        terminfo: &TermInfoItem<'w, 's>,
        grid: &mut Grid,
        WriteParams {
            text,
            cursor,
            style,
        }: WriteParams,
    ) {
        // get current line and offset from cursor
        // add to that line's buffer
        if let Some((line_ref, spans)) = grid.get(cursor.line) {
            let mut lineval = line_ref.value().value.clone();
            lineval.insert_str(cursor.char, text.as_str());
            let new_line = TerminalLine::new(terminfo.id, lineval);
            let mut offset_increment = 0;
            let spans = spans
                .iter()
                .flat_map(|span_ref| {
                    let span_val = span_ref.value();
                    // offset <= cursor.char <= offset + range
                    if span_val.offset <= cursor.char
                        && cursor.char <= span_val.offset + span_val.range
                    {
                        // if style doesn't match, then split the range and insert between
                        if span_val.style != style {
                            let this_val = VirtualTextSpan {
                                range: span_val.range - new_line.len(),
                                ..*span_val
                            };
                            let new_val = VirtualTextSpan {
                                range: new_line.len(),
                                offset: this_val.offset + this_val.range,
                                style,
                                line: this_val.line,
                            };
                            offset_increment += 1;
                            vec![
                                MaybeRef::Owned(span_ref.entity(), this_val),
                                MaybeRef::Owned(span_ref.entity(), new_val),
                            ]
                        } else {
                            // otherwise modify the span
                            let new_span = VirtualTextSpan {
                                line: Entity::PLACEHOLDER,
                                range: span_val.range + new_line.len(),
                                ..*span_val
                            };
                            vec![MaybeRef::Owned(None, new_span)]
                        }
                    } else if offset_increment != 0 {
                        let new_span = VirtualTextSpan {
                            offset: span_val.offset + offset_increment,
                            ..*span_val
                        };
                        vec![MaybeRef::Owned(span_ref.entity(), new_span)]
                    } else {
                        vec![span_ref.clone()]
                    }
                })
                .collect::<Vec<_>>();
            grid.insert(
                cursor.line,
                (MaybeRef::Owned(line_ref.entity(), new_line), spans),
            );
        } else {
            let line = TerminalLine::new(
                terminfo.id,
                String::from_iter((0..cursor.char).map(|_| ' ')) + text.as_str(),
            );
            let span = VirtualTextSpan {
                line: Entity::PLACEHOLDER,
                offset: 0,
                range: text.len(),
                style,
            };
            if grid.len() < cursor.line {
                grid.resize_with(cursor.line, || {
                    (
                        MaybeRef::Owned(None, TerminalLine::new(terminfo.id, String::new())),
                        vec![MaybeRef::Owned(
                            None,
                            VirtualTextSpan {
                                line: Entity::PLACEHOLDER,
                                offset: 0,
                                range: 0,
                                style,
                            },
                        )],
                    )
                });
            }
            grid.insert(
                cursor.line,
                (
                    MaybeRef::Owned(None, line),
                    vec![MaybeRef::Owned(None, span)],
                ),
            );
        };
    }

    pub fn execute(
        self,
        commands: &mut Commands,
        q_lines: &Query<(Entity, &TerminalLine)>,
        q_spans: &Query<(Entity, &VirtualTextSpan)>,
    ) {
        // TODO: Iterate through self.actions and perform commands to spawn
        // and modify entities as appropriate
        // Spawn virutal text spans and their corresponding lines.
        // Then flow the lines per window.
        let terminfo = self.terminfo;
        let mut grid: Grid = self
            .terminfo
            .grid(q_lines, q_spans)
            .map(|(line_id, line, spans)| {
                (
                    MaybeRef::Borrowed(line_id, line),
                    spans
                        .into_iter()
                        .map(|(span_id, span)| MaybeRef::Borrowed(span_id, span))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        // update cursor to final position
        commands.entity(terminfo.id).insert(self.cursor);
        // update grid entities
        for params in self.to_write.into_iter() {
            Self::write(terminfo, &mut grid, params);
        }
        for (line, spans) in grid.into_iter() {
            let line_id = match line {
                MaybeRef::Owned(Some(entity), line) => commands.entity(entity).insert(line).id(),
                MaybeRef::Owned(None, line) => commands.spawn(line).id(),
                _ => continue,
            };
            for span in spans {
                // TODO: consolidate spans
                match span {
                    MaybeRef::Owned(None, span) => {
                        let span_id = commands
                            .spawn(VirtualTextSpan {
                                line: line_id,
                                ..span
                            })
                            .id();
                        commands.entity(line_id).add_child(span_id);
                    }
                    MaybeRef::Owned(Some(id), span) => {
                        commands.entity(id).insert(VirtualTextSpan {
                            line: line_id,
                            ..span
                        });
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
impl<'w, 's, 'a> anstyle_parse::Perform for AnsiPerformer<'w, 's, 'a> {
    fn print(&mut self, c: char) {
        info!(?self.cursor, ?c);
        self.to_write.push(WriteParams {
            text: c.to_smolstr(),
            style: self.style,
            cursor: self.cursor,
        });
        self.cursor.char += 1;
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            byte if byte == ControlCodes::BEL as u8 => {
                // TODO: sound a bell :)
                info!(?self.cursor, "BEL");
            }
            byte if byte == ControlCodes::LF as u8 => {
                self.cursor.line += 1;
                self.cursor.char = 0;
                info!(?self.cursor, "LF");
            }
            byte if byte == ControlCodes::CR as u8 => {
                self.cursor.char = 0;
                info!(?self.cursor, "CR");
            }
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
        info!(?_action, "hook");
    }

    fn put(&mut self, _byte: u8) {
        info!(?_byte, "put");
    }

    fn unhook(&mut self) {
        info!("(unhook)")
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    // FUN FACT! THIS IS NOT CORRECT :)
    // Terminal cursor actions must account for text wrapping,
    // since it is relative to the _displayed_ grid
    fn csi_dispatch(
        &mut self,
        params: &anstyle_parse::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: u8,
    ) {
        info!(?action, "csi_dispatch");
        let mut param_iter = params.iter();
        match action {
            action if action == CsiAction::CUU as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.line -= *next as usize;
            }
            action if action == CsiAction::CUD as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.line += *next as usize;
            }
            action if action == CsiAction::CUF as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.char += *next as usize;
            }
            action if action == CsiAction::CUB as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.char -= *next as usize;
            }
            action if action == CsiAction::CNL as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.line += *next as usize;
                self.cursor.char = 0;
            }
            action if action == CsiAction::CPL as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.line -= *next as usize;
                self.cursor.char = 0;
            }
            action if action == CsiAction::CHA as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor.char = *next as usize;
            }
            action if action == CsiAction::CUP as u8 || action == CsiAction::HVP as u8 => {
                let [line, char, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.cursor = TerminalCursor {
                    line: *line as usize,
                    char: *char as usize,
                };
            }
            // action if action == CsiAction::ED as u8 => {
            //     self.actions.push(AnsiAction::Erase { mode: iter.next().unwrap_or(0) as usize });
            // }
            // action if action == CsiAction::EL as u8 => {
            //     self.actions.push(AnsiAction::Erase { mode: iter.next().unwrap_or(0) as usize });
            // }
            action if action == CsiAction::SGR as u8 => {
                match param_iter.next() {
                    Some([0]) => self.style = self.default_style,
                    Some([8]) => info_once!("Conceal color mode not yet implemented"),
                    // palette mode
                    Some([x])
                        if (*x >= 30 && *x <= 37)
                            || (*x >= 40 && *x <= 47)
                            || (*x >= 90 && *x <= 97)
                            || (*x >= 100 && *x <= 107) =>
                    {
                        let is_dark = x / 10 == 3 || x / 10 == 4;
                        let is_bg = x / 10 == 4 || x / 10 == 10;
                        if x % 10 == 9 {
                            if is_bg {
                                self.style.background = self.default_style.background;
                            } else {
                                self.style.color = self.default_style.color;
                            }
                            return;
                        }
                        // todo : TerminalPalette component or resource - per window or global?
                        let color = match (x % 10, is_dark) {
                            (0, _) => css::BLACK,
                            (1, false) => css::RED,
                            (1, true) => css::DARK_RED,
                            (2, false) => css::GREEN,
                            (3, true) => css::DARK_GREEN,
                            (4, false) => css::LIGHT_YELLOW,
                            (4, true) => css::YELLOW,
                            (5, false) => css::MAGENTA,
                            (5, true) => css::DARK_MAGENTA,
                            (6, false) => css::LIGHT_CYAN,
                            (6, true) => css::DARK_CYAN,
                            (7, false) => css::WHITE,
                            (7, true) => css::GRAY,
                            _ => {
                                unreachable!()
                            }
                        };
                        if is_bg {
                            self.style.background = color.into();
                        } else {
                            self.style.color = color.into();
                        }
                    }
                    // 256 color mode
                    Some([38, 5, _n]) => {
                        info_once!("256 color mode not currently supported.")
                    }
                    // 24-bit color
                    // TODO?: x == 58 for underline styling
                    Some([x, 2, r, g, b]) if *x == 38 || *x == 48 => {
                        let color = Color::srgb_u8(*r as u8, *g as u8, *b as u8);
                        if *x == 38 {
                            self.style.color = color;
                        } else {
                            self.style.background = color;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
