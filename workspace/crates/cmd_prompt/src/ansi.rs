#![allow(clippy::upper_case_acronyms)]

use std::collections::VecDeque;

use crate::prelude::*;
use anstyle_parse::{Parser, Utf8Parser};
use bevy::color::palettes::css;
use itertools::Itertools;

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

#[derive(Debug, Clone, PartialEq)]
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

macro_rules! assert_cursor_in_view {
    ($self:ident$(, $retvalue:expr)?) => {
        assert!($self.cursor.line <= $self.viewport_rows);
        assert!($self.cursor.char <= $self.viewport_cols);
    };
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct VisibleRowIndex {
    line_idx: usize,
    row_idx: usize,
}

#[derive(Debug, PartialEq, Clone)]
struct GridLine<'a> {
    line: MaybeRef<'a, VtLine>,
    rows: Vec<GridRow<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
struct GridRow<'a> {
    row: MaybeRef<'a, VtRow>,
    visible: bool,
}

#[derive(Debug, PartialEq, Clone)]
struct GridViewportRow<'a> {
    vrow: MaybeRef<'a, VtViewportRow>,
    has_row: bool,
}

/// A transient grid. Used to modify the terminal entities and reconstructed
/// on each pass.
#[derive(Debug)]
pub struct Grid<'a> {
    lines: Vec<GridLine<'a>>,
    viewport_cols: usize,
    viewport_rows: usize,
    scroll_pos: usize,
    cursor: VtCursor,
    term_id: Entity,
}
impl<'a> Grid<'a> {
    pub fn new<'w, 's>(
        terminfo: &TermInfoItem<'w, 's>,
        q_lines: &'a Query<(Entity, &VtLine, &VtRowTarget)>,
        q_rows: &'a Query<(Entity, &VtRow, Option<&VtViewportRow>)>,
    ) -> Self {
        let lines = q_lines
            .iter_many(terminfo.line_target.entities())
            .map(|(line_id, line, row_target)| {
                let rows = q_rows
                    .iter_many(row_target.entities())
                    .map(|(row_id, row, maybe_vrow)| GridRow {
                        row: MaybeRef::Borrowed(row_id, row),
                        visible: maybe_vrow.is_some(),
                    })
                    .collect::<Vec<_>>();
                GridLine {
                    line: MaybeRef::Borrowed(line_id, line),
                    rows,
                }
            })
            .collect::<Vec<_>>();

        Self {
            lines,
            viewport_cols: terminfo.size.cols,
            viewport_rows: terminfo.size.rows,
            scroll_pos: terminfo.scroll_pos.0,
            cursor: *terminfo.cursor,
            term_id: terminfo.id,
        }
    }

    pub fn scroll_viewport(&mut self, dir: isize) {
        self.scroll_pos = self.scroll_pos.saturating_add_signed(dir);
    }

    pub fn increment_char(&mut self, wrap: bool) {
        self.cursor.char = (self.cursor.char + 1).max(self.viewport_cols);
        if wrap && self.cursor.char >= self.viewport_cols {
            if self.cursor.pending_wrap {
                self.cursor.char = 0;
                self.increment_line(wrap);
            } else {
                self.cursor.pending_wrap = true;
            }
        }
        assert_cursor_in_view!(self);
    }
    pub fn decrement_char(&mut self, wrap: bool) {
        if self.cursor.char == 0 && wrap {
            self.decrement_line(true);
            self.cursor.char = self.viewport_cols;
            self.cursor.pending_wrap = true;
        } else {
            self.cursor.char = self.cursor.char.saturating_sub(1);
        }
        assert_cursor_in_view!(self);
    }
    pub fn increment_line(&mut self, scroll: bool) {
        self.cursor.line = (self.cursor.line + 1).max(self.viewport_rows);
        if scroll && self.cursor.line >= self.viewport_rows {
            self.scroll_pos += 1;
            self.cursor.line -= 1;
        }
        assert_cursor_in_view!(self);
    }
    pub fn decrement_line(&mut self, scroll: bool) {
        if scroll && self.cursor.line == 0 {
            self.scroll_pos -= 1;
        } else {
            self.cursor.char = self.cursor.line.saturating_sub(1);
        }
        assert_cursor_in_view!(self);
    }

    /// Returns (line_idx, row_idx) in _reverse order._
    /// row_idx is relative to the line
    fn visible_rows(&self) -> VecDeque<VisibleRowIndex> {
        assert_cursor_in_view!(self, vec![]);
        self.lines
            .iter()
            .enumerate()
            .flat_map(|(line_idx, line)| {
                line.rows
                    .iter()
                    .enumerate()
                    .map(|(row_idx, _)| VisibleRowIndex { line_idx, row_idx })
                    .collect::<Vec<_>>()
            })
            .rev()
            .skip(self.scroll_pos)
            .take(self.viewport_rows)
            .collect::<VecDeque<_>>()
    }

    pub fn write(&mut self, c: char, style: VtCellStyle) {
        trace!("Write");
        let mut visible_rows = self.visible_rows();
        while visible_rows.len() < self.cursor.line {
            let new_line = GridLine {
                line: MaybeRef::Owned(None, VtLine::new(self.term_id)),
                rows: vec![GridRow {
                    row: MaybeRef::Owned(None, VtRow::new(Entity::PLACEHOLDER, 0)),
                    visible: true,
                }],
            };
            self.lines.push(new_line);
            visible_rows.push_back(VisibleRowIndex {
                line_idx: self.lines.len(),
                row_idx: 0,
            });
        }
        trace!(?visible_rows, ?self.cursor);

        let VisibleRowIndex { line_idx, row_idx } = r!(visible_rows.get(self.cursor.line));
        let gridline = self.lines.get_mut(*line_idx).unwrap();
        let gridrow = gridline.rows.get_mut(*row_idx).unwrap();
        let pos = gridrow.row.value().offset + self.cursor.char;
        // update line cells
        let mut cells = gridline.line.value().cells().to_vec();
        cells.insert(pos, VtCell::new(c).with_style(style));
        gridline.line = MaybeRef::Owned(
            gridline.line.entity(),
            VtLine::from_cells(self.term_id, cells),
        );
        // bump following offsets
        gridline
            .rows
            .iter_mut()
            .enumerate()
            .for_each(|(idx, gridrow)| {
                if idx > *row_idx {
                    let line_id = gridrow.row.value().line();
                    let offset = gridrow.row.value().offset + 1;
                    gridrow.row = MaybeRef::Owned(gridrow.row.entity(), VtRow::new(line_id, offset))
                }
            });
        trace!("{gridline:#?}");
    }

    /// Synchronize the [`Grid`] with the [`Terminal`] component in the
    /// [`World`]. Will update the corresponding [`VtRow`], [`VtLine`],
    /// [`VtCursor`], [`VtScrollPos`], and other components.
    pub fn sync(self, commands: &mut Commands) {
        debug!("{self:#?}");
        let visible_rows = self.visible_rows();
        // cache viewport info
        commands
            .entity(self.term_id)
            .insert((self.cursor, VtScrollPos(self.scroll_pos)));
        // update grid entities
        for (line_idx, gridline) in self.lines.into_iter().enumerate() {
            let line_id = match gridline.line {
                MaybeRef::Owned(Some(entity), line) => commands.entity(entity).insert(line).id(),
                MaybeRef::Owned(None, line) => commands.spawn(line).id(),
                _ => continue,
            };
            gridline
                .rows
                .into_iter()
                .enumerate()
                .for_each(|(row_idx, gridrow)| {
                    let rowid = match gridrow.row {
                        MaybeRef::Owned(Some(entity), row) => commands
                            .entity(entity)
                            .insert(VtRow::new(line_id, row.offset))
                            .id(),
                        MaybeRef::Owned(None, row) => {
                            commands.spawn(VtRow::new(line_id, row.offset)).id()
                        }
                        _ => return,
                    };
                    if visible_rows.contains(&VisibleRowIndex { line_idx, row_idx }) {
                        commands
                            .entity(rowid)
                            .insert(VtViewportRow::new(self.term_id));
                    }
                });
        }
    }
}

/// Parses the passed [`VirtualTextSpanSpawner`], possibly expanding it into multiple
/// and modifying various [`TerminalLine`]s.
pub(crate) struct AnsiPerformer<'a, 'g> {
    grid: &'a mut Grid<'g>,
    style: VtCellStyle,
    default_style: VtCellStyle,
}
impl<'a, 'g> AnsiPerformer<'a, 'g> {
    pub fn new(grid: &'a mut Grid<'g>) -> Self {
        Self {
            grid,
            style: VtCellStyle::default(),
            default_style: VtCellStyle::default(),
        }
    }
    pub fn set_default_style(&mut self, style: VtCellStyle) {
        self.default_style = style;
    }
}
impl<'a, 'g> anstyle_parse::Perform for AnsiPerformer<'a, 'g> {
    fn print(&mut self, c: char) {
        self.grid.write(c, self.style);
        self.grid.increment_char(true);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            byte if byte == ControlCodes::BEL as u8 => {
                // TODO: sound a bell :)
                info!(?self.grid, "BEL");
            }
            byte if byte == ControlCodes::LF as u8 => {
                self.grid.increment_line(true);
            }
            byte if byte == ControlCodes::CR as u8 => {
                self.grid.cursor.char = 0;
            }
            _ => {
                info_once!(
                    "Given an unsupported escape C0 or C1 escape sequence. See the docs supported types."
                );
                // unsupported
            }
        }
    }

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
                for _ in 0..*next {
                    self.grid.decrement_line(false);
                }
            }
            action if action == CsiAction::CUD as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                for _ in 0..*next {
                    self.grid.increment_line(false);
                }
            }
            action if action == CsiAction::CUF as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                for _ in 0..*next {
                    self.grid.increment_char(false);
                }
            }
            action if action == CsiAction::CUB as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                for _ in 0..*next {
                    self.grid.decrement_char(false);
                }
            }
            action if action == CsiAction::CNL as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.grid.cursor.char = 0;
                for _ in 0..*next {
                    self.grid.increment_line(false);
                }
            }
            action if action == CsiAction::CPL as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.grid.cursor.char = 0;
                for _ in 0..*next {
                    self.grid.decrement_line(false);
                }
            }
            action if action == CsiAction::CHA as u8 => {
                let [next, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.grid.cursor.char = (*next as usize).clamp(0, self.grid.viewport_cols);
            }
            action if action == CsiAction::CUP as u8 || action == CsiAction::HVP as u8 => {
                let [line, char, ..] = param_iter.next().unwrap_or(&[0u16]) else {
                    return;
                };
                self.grid.cursor.line = (*line as usize).clamp(0, self.grid.viewport_rows);
                self.grid.cursor.char = (*char as usize).clamp(0, self.grid.viewport_cols);
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
