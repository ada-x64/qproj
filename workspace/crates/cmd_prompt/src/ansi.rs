use crate::prelude::*;
use anstyle_parse::{Parser, Utf8Parser};
use bevy::{color::palettes::css, platform::collections::HashMap};
use smol_str::{SmolStr, ToSmolStr};

pub type AnsiParser = Parser<Utf8Parser>;

enum AnsiAction {
    Write {
        text: SmolStr,
    },
    SetCursorPos {
        line: usize,
        char: usize,
    },
    Erase {
        // TODO - Can only support this partially,
        // since we're not directly modifying the displayed grid
    },
    ResetStyle,
    ResetColor,
    ResetBackground,
    SetColor(Color),
    SetBackground(Color),
    Conceal,
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

#[derive(Clone, Copy)]
pub struct Style {
    pub color: Color,
    pub background: Color,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            background: Color::BLACK,
        }
    }
}

/// Parses the passed [`VirtualTextSpanSpawner`], possibly expanding it into multiple
/// and modifying various [`TerminalLine`]s.
pub(crate) struct AnsiPerformer<'w, 's, 'a, 't> {
    terminfo: &'a TermInfoItem<'w, 's, 't>,
    cursor: TerminalCursor,
    new_lines: Vec<NewItem<TerminalLine>>,
    new_vspans: Vec<NewItem<VirtualTextSpan>>,
    actions: Vec<AnsiAction>,
    current_style: Style,
    default_style: Style,
}
impl<'w, 's, 'a, 't> AnsiPerformer<'w, 's, 'a, 't> {
    pub fn new(terminfo: &'a TermInfoItem<'w, 's, 't>, default_style: Style) -> Self {
        Self {
            terminfo,
            cursor: *terminfo.cursor,
            actions: vec![],
            new_lines: vec![],
            new_vspans: vec![],
            current_style: default_style,
            default_style,
        }
    }
    /// Returns vectors of new components to add or replace. If the optional
    /// entity is Some, then the result is to be replaced.
    // cursor actions must happen _per viewport so we need to translate from
    // window cursor position to terminal cursor position by accounting for
    // the scroll offset and terminal size
    pub fn execute(mut self, commands: &mut Commands) {
        // TODO: Iterate through self.actions and perform commands to spawn
        // and modify entities as appropriate
        // Spawn virutal text spans and their corresponding lines.
        // Then flow the lines per window.
        for action in self.actions {
            match action {
                AnsiAction::Write { text } => {
                    // get current line and offset from cursor
                    // add to that line's buffer
                    if let Some(e) = lines.get(self.cursor.line)
                        && let Some(line) = q_lines.get(e)
                    {
                        let mut value = line.value.clone();
                        value.insert_str(self.cursor.char, text.as_str());
                        let line = TerminalLine::new(terminal, value);
                        commands.entity(*e).insert(line);
                    } else {
                        let line = TerminalLine::new(
                            terminal,
                            String::from_iter((0..self.cursor.char).map(|_| ' ')) + text.as_str(),
                        );
                        commands.spawn(line);
                    };
                }
                // ANSI grid coordinates are 1-indexed.
                AnsiAction::SetCursorPos { line, char } => {
                    self.cursor.line = line.saturating_sub(1);
                    self.cursor.char = char.saturating_sub(1);
                }
                AnsiAction::Erase {} => todo!(),
                AnsiAction::ResetStyle => {
                    self.current_style = self.default_style;
                }
                AnsiAction::ResetColor => {
                    self.current_style.color = self.default_style.color;
                }
                AnsiAction::ResetBackground => {
                    self.current_style.background = self.default_style.background;
                }
                AnsiAction::SetColor(color) => {
                    self.current_style.color = color;
                }
                AnsiAction::SetBackground(color) => {
                    self.current_style.background = color;
                }
                AnsiAction::Conceal => todo!(),
            }
        }
    }
}
impl<'w, 's, 'a, 't> anstyle_parse::Perform for AnsiPerformer<'w, 's, 'a, 't> {
    fn print(&mut self, c: char) {
        self.actions.push(AnsiAction::Write {
            text: c.to_smolstr(),
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
        let mut param_iter = params.iter();
        match action {
            action if action == CsiAction::CUU as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line - (*next as usize),
                    char: self.cursor.char,
                });
            }
            action if action == CsiAction::CUD as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line + (*next as usize),
                    char: self.cursor.char,
                });
            }
            action if action == CsiAction::CUF as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line,
                    char: self.cursor.char + (*next as usize),
                })
            }
            action if action == CsiAction::CUB as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line,
                    char: self.cursor.char - (*next as usize),
                });
            }
            action if action == CsiAction::CNL as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line + *next as usize,
                    char: 0,
                });
            }
            action if action == CsiAction::CPL as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line - (*next as usize),
                    char: 0,
                });
            }
            action if action == CsiAction::CHA as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: self.cursor.line,
                    char: *next as usize,
                });
            }
            action if action == CsiAction::CUP as u8 || action == CsiAction::HVP as u8 => {
                let [line, char, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.actions.push(AnsiAction::SetCursorPos {
                    line: *line as usize,
                    char: *char as usize,
                });
            }
            // action if action == CsiAction::ED as u8 => {
            //     self.actions.push(AnsiAction::Erase { mode: iter.next().unwrap_or(0) as usize });
            // }
            // action if action == CsiAction::EL as u8 => {
            //     self.actions.push(AnsiAction::Erase { mode: iter.next().unwrap_or(0) as usize });
            // }
            action if action == CsiAction::SGR as u8 => {
                // parse into correct format
                // TODO: Style stuff with anstyle
                match param_iter.next() {
                    Some([0]) => self.actions.push(AnsiAction::ResetStyle),
                    Some([8]) => self.actions.push(AnsiAction::Conceal),
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
                                self.actions.push(AnsiAction::ResetBackground);
                            } else {
                                self.actions.push(AnsiAction::ResetColor);
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
                        self.actions.push(AnsiAction::SetColor(color.into()))
                    }
                    // 256 color mode
                    Some([38, 5, n]) => {
                        info_once!("256 color mode not currently supported.")
                    }
                    // 24-bit color
                    // TODO?: x == 58 for underline styling
                    Some([x, 2, r, g, b]) if *x == 38 || *x == 48 => {
                        let color = Color::srgb_u8(*r as u8, *g as u8, *b as u8);
                        if *x == 38 {
                            self.actions.push(AnsiAction::SetColor(color));
                        } else {
                            self.actions.push(AnsiAction::SetBackground(color));
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
