use crate::prelude::*;

mod terminfo {
    use super::*;
    use bevy::ecs::query::QueryData;

    /// Public API and query helpers for the [`Terminal`] entity.
    #[derive(QueryData, Debug)]
    pub struct TermInfo {
        pub id: Entity,
        pub cursor: &'static VtCursor,
        pub(crate) lines: &'static VtLineTarget,
        pub(crate) viewport: &'static VtViewport,
        pub size: &'static VtSize,
        pub scroll_pos: &'static VtScrollPos,
    }
    impl<'w, 's> TermInfoItem<'w, 's> {
        #[inline(always)]
        pub fn lines<'a>(
            &self,
            q_lines: &'a Query<(Entity, &VtLine)>,
        ) -> impl Iterator<Item = (Entity, &'a VtLine)> {
            q_lines.iter_many(self.lines.iter())
        }

        #[inline(always)]
        pub fn viewport_rows<'a>(
            &self,
            q_viewport_rows: &'a Query<(Entity, &VtViewportRow)>,
        ) -> impl Iterator<Item = (Entity, &'a VtViewportRow)> {
            q_viewport_rows.iter_many(self.viewport.iter())
        }

        pub fn rows<'a>(
            &self,
            q_row_targets: &'a Query<&VtRowTarget, With<VtLine>>,
            q_rows: &'a Query<(Entity, &VtRow, Option<&VtViewportRow>)>,
        ) -> impl Iterator<Item = (Entity, &'a VtRow, Option<&'a VtViewportRow>)> {
            self.lines.iter().flat_map(|line_id| {
                let target = r!(q_row_targets.get(line_id).ok());
                q_rows.iter_many(target.entities()).collect::<Vec<_>>()
            })
        }

        #[inline(always)]
        pub fn mutate(&self, commands: &mut Commands, msg: TermMsgKind) {
            commands.write_message(TermMsg::new(self.id, msg));
        }
    }
}
pub use terminfo::*;

mod pty {
    use super::*;

    /// Pseudo-terminal entity marker. This component is in charge
    /// of storing the input buffer and sending data from the [`PtyLeader`]
    /// (the terminal) to the [`PtyFollower`] (the shell).
    #[derive(Component, Default, Debug, Reflect)]
    #[require(
        PtyLeader(Entity::PLACEHOLDER),
        PtyFollower(Entity::PLACEHOLDER),
        LineDiscipline
    )]
    pub struct Pty {
        /// The input buffer.
        pub buffer: String,
    }

    /// How the [`Terminal`] sends information to the [`TerminalShell`].
    /// Canonical mode is the default. It sends lines on submit.
    /// Raw mode sends inputs unbuffered. This is useful for TUIs like vim or htop.
    #[derive(Component, Default, Reflect, Debug)]
    #[component(immutable)]
    pub enum LineDiscipline {
        #[default]
        Canonical,
        Raw,
    }

    #[derive(Component, Reflect, Debug)]
    pub struct PtyLeader(Entity);

    #[derive(Component, Reflect, Debug)]
    pub struct PtyFollower(Entity);
}
pub use pty::*;

mod terminal {
    use super::*;

    use bevy::{
        ecs::{lifecycle::HookContext, world::DeferredWorld},
        picking::hover::DirectlyHovered,
        text::LineHeight,
    };

    /// A terminal display. Will spawn a new [`Node`] sized to the parent
    /// container and populate the hierarchy with [`TextSpan`] components according
    /// to the target entity's properties.
    ///
    /// Related components are prefixed with `Vt`, e.g. [`VtLayout`]
    #[derive(Component, Default, Reflect)]
    #[require(
        // Interal
        VtLineTarget,
        VtCursor,
        VtScrollPos,
        VtSize,
        VtViewport,
        // Display items
        Node {
            overflow: Overflow::clip(),
            ..Default::default()
        },
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::NoWrap
        },
        Pickable,
        TextColor,
        TextFont,
        LineHeight,
        )]
    #[component(immutable, on_add=Self::on_add)]
    pub struct Terminal;
    impl Terminal {
        /// Spawn [`TerminalCharWidth`] et al
        pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            let mut commands = world.commands();
            let id = commands.spawn(VtCharWidth::new(ctx.entity, 0.)).id();
            commands
                .entity(ctx.entity)
                .add_one_related::<VtCharWidth>(id)
                .observe(on_scroll)
                .observe(|event: On<Pointer<Over>>| {
                    debug!(?event);
                });
        }
    }

    /// Cursor for the [`Terminal`]. Points at a given char index into a
    /// [`TerminalRow`] (nth from end). Note that this is relative to the
    /// _viewport_.
    ///
    /// Cursor positions are relative to the top-left of the viewport, with x
    /// increasing to the right and y increasing downwards. This is the opposite
    /// of the [`TerminalRow`] index.
    ///
    /// ```notrust
    ///                        row idx | cursor line
    /// (0,0).---------.             3 | 0
    ///      |         |             2 | 1
    ///      |         |             1 | 2
    ///      `---------`(10,3)       0 | 3
    /// ```
    #[derive(Component, Default, Reflect, Clone, Copy, Debug)]
    pub struct VtCursor {
        pub line: usize,
        pub char: usize,
        pub pending_wrap: bool,
    }
    impl VtCursor {
        pub fn new(line: usize, char: usize) -> Self {
            Self {
                line,
                char,
                pending_wrap: false,
            }
        }
    }

    // TODO: Add a limit to the number of stored lines.
    // This can't currently be a vecdeque due to trait constraints,
    // so need to add a manual 'maximum' field and do checks on insert
    // to ensure the vec doesn't overload its capacity.
    /// This entity represents the underlying text buffer.
    ///
    /// Relationship target for [`VtLine`] 1:n
    #[derive(Component, Default, Deref, Debug, Reflect)]
    #[relationship_target(relationship=VtLine)]
    pub struct VtLineTarget(Vec<Entity>);

    /// A single, newline-delimited logical line. Lines are composed of vecs of
    /// [`VtCell`]s. Lines do _not_ include the trailing newline.
    ///
    /// This component requires [`VtRowTarget`]. All [`VtRow`]s related to this
    /// entity can be accessed through that relationship. In this way, updates
    /// to [`VtLine`] components will propogate to the visible lines.
    ///
    /// ```notrust
    ///     (VtLineTarget) -> (VtLine; VtRowTarget) -> (VtRow)
    ///                 |                         \--> (VtRow)
    ///                 \---> (VtLine; VtRowTarget) -> (VtRow; VtViewportRow)
    ///                                                         V
    ///                                            (Terminal; VtLayout)
    /// ```
    ///
    /// Related to [`VtLineTarget`] 1:n
    #[derive(Component, Debug, Reflect, PartialEq, Clone)]
    #[relationship(relationship_target=VtLineTarget)]
    #[require(VtRowTarget)]
    pub struct VtLine {
        pub value: Vec<VtCell>,
        #[relationship]
        target: Entity,
    }
    impl VtLine {
        pub fn new(terminal_id: Entity) -> Self {
            Self {
                value: vec![],
                target: terminal_id,
            }
        }
        pub fn from_str<S: ToString>(terminal_id: Entity, string: S) -> Self {
            Self {
                value: string.to_string().chars().map(|c| VtCell::new(c)).collect(),
                target: terminal_id,
            }
        }
        pub fn from_str_with_style<S: ToString>(
            terminal_id: Entity,
            string: S,
            style: VtCellStyle,
        ) -> Self {
            Self {
                value: string
                    .to_string()
                    .chars()
                    .map(|c| VtCell::new(c).with_style(style))
                    .collect(),
                target: terminal_id,
            }
        }
        pub fn as_string(&self) -> String {
            self.value.iter().map(|cell| cell.value).collect()
        }
        pub fn from_cells(terminal_id: Entity, cells: Vec<VtCell>) -> Self {
            Self {
                value: cells,
                target: terminal_id,
            }
        }
    }

    /// Always coupled with [`VtLine`].
    ///
    /// Relationship target for [`VtRow`] 1:n
    #[derive(Component, Debug, Reflect, Default)]
    #[relationship_target(relationship=VtRow)]
    pub struct VtRowTarget(Vec<Entity>);
    impl VtRowTarget {
        pub fn entities(&self) -> &[Entity] {
            &self.0
        }
    }

    /// A single cell within a [`VtLine`]. The basic building block of a virtual
    /// terminal.
    #[derive(Debug, Reflect, PartialEq, Clone, Copy)]
    pub struct VtCell {
        pub value: char,
        pub style: VtCellStyle,
    }
    impl VtCell {
        pub fn new(value: char) -> Self {
            Self {
                value,
                style: VtCellStyle::default(),
            }
        }
        pub fn with_style(mut self, style: VtCellStyle) -> Self {
            self.style = style;
            self
        }
    }

    /// Text styles for [`VtCell`]s.
    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    pub struct VtCellStyle {
        pub color: Color,
        pub background: Color,
    }
    impl Default for VtCellStyle {
        fn default() -> Self {
            Self {
                color: Color::WHITE,
                background: Color::BLACK,
            }
        }
    }

    /// Scroll position, in lines. 0 means you're at the bottom.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref, Default)]
    #[component(immutable)]
    pub struct VtScrollPos(pub usize);

    /// Width of a character cell in pixels, determined by measuring the width of a space.
    /// Related to [`VtCharWidthTarget`] (1:1)
    #[derive(Component, Debug, Reflect, PartialEq, Clone, Copy)]
    #[component(immutable)]
    #[require(Node::default(), Pickable::IGNORE, Visibility::Hidden, Text::new(" "))]
    #[relationship(relationship_target=VtCharWidthTarget)]
    pub struct VtCharWidth {
        #[relationship]
        target: Entity,
        value: f32,
    }
    impl VtCharWidth {
        pub fn new(target: Entity, value: f32) -> Self {
            Self { target, value }
        }
        pub fn value(&self) -> f32 {
            self.value
        }
        pub fn target(&self) -> Entity {
            self.target
        }
    }

    /// Relationship target for [`VtCharWidth`]. Relationship is 1:1.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref)]
    #[relationship_target(relationship=VtCharWidth, linked_spawn)]
    pub struct VtCharWidthTarget(Entity);
    impl VtCharWidthTarget {
        pub fn target(&self) -> Entity {
            self.0
        }
    }

    /// Visible layout for the terminal.
    /// The terminal's layout grid. Contains references to entities which should
    /// contain [`VtRow`] and [`VtViewportRow`] components.
    ///
    /// Relationship target for [`VtViewportRow`] (1:n).
    #[derive(Component, Default, Deref, Debug, Reflect)]
    #[relationship_target(relationship=VtViewportRow)]
    #[component(on_add = Self::on_add)]
    pub struct VtViewport(Vec<Entity>);
    impl VtViewport {
        fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            let child_node = world.commands().spawn(VtTextWrapper).id();
            world.commands().entity(ctx.entity).add_child(child_node);
        }
    }

    /// A marker component for the [`VtLayout`] entity's child component,
    /// which contains the automatically created [`TextSpan`] children;
    #[derive(Component, Default, Debug, Reflect)]
    #[require(Node, Text, DirectlyHovered, Pickable)]
    pub struct VtTextWrapper;

    /// Visible row. Possible sibling of [`VtRow`]. Automatically spawns
    /// necessary components to render this row. See [`VtRow`] for more details.
    ///
    /// This entity does not require [`VtRow`] as there may be empty terminal
    /// lines that still need to be renered, for example when the buffer is
    /// cleared.
    ///
    /// Related to [`VtViewport`] 1:n.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy)]
    #[relationship(relationship_target = VtViewport)]
    #[component(immutable)]
    pub struct VtViewportRow(Entity);
    impl VtViewportRow {
        pub fn new(term_id: Entity) -> Self {
            Self(term_id)
        }
        pub fn on_insert() {
            // spawn node et al
        }
    }

    /// A reference to a logical line with a character offset into its full text
    /// value.
    ///
    /// This entity may optionally have a [`VtViewportRow`] component attached
    /// to it. If so, it will automatically spawn the necessary components to
    /// render this row to the viewport.
    ///
    /// ```notrust
    ///                              (Terminal; VtViewport)
    ///                                             ^
    ///     (VtLine; VtRowTarget) -> (VtRow; VtViewportRow; Children)
    ///                      \-----> (VtRow)                   \-> (Node; Text; ...)
    /// ```
    ///
    /// Related to [`VtLayout`] (1:n)
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy)]
    #[relationship(relationship_target=VtRowTarget)]
    #[component(immutable)]
    pub struct VtRow {
        /// Character offset into the line at which to begin this span.
        /// The range is always = [`VtSize::cols`].
        pub offset: usize,
        /// Relationship target. Points to `VtRowTarget`.
        #[relationship]
        line: Entity,
    }
    impl VtRow {
        pub fn new(line: Entity, offset: usize) -> Self {
            Self { offset, line }
        }

        pub fn line(&self) -> Entity {
            self.line
        }
    }

    #[derive(Component, Default, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct VtSize {
        pub rows: usize,
        pub cols: usize,
    }
    impl VtSize {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            world.commands().write_message(TermMsg::reflow(ctx.entity));
        }
    }
}
pub use terminal::*;

mod events {
    use super::*;

    /// A mutation to send to the terminal window.
    #[derive(Debug, Message, Reflect)]
    pub struct TermMsg {
        pub target: Entity,
        pub kind: TermMsgKind,
    }
    #[derive(Debug, Reflect)]
    pub enum TermMsgKind {
        /// Reflow the terminal. Modifes LineRefs.
        Reflow,
        /// Scroll in the given direction. A scroll position of 0 means you are
        /// at the last line.
        Scroll(isize),
        /// Jump to the last line. Sets scroll value to 0.
        JumpToBottom,
        /// Write charaters to the screen.
        Write(Vec<TermWrite>),
    }
    impl TermMsg {
        pub fn new(window_id: Entity, kind: TermMsgKind) -> Self {
            Self {
                target: window_id,
                kind,
            }
        }
        pub fn scroll(window_id: Entity, direction: isize) -> Self {
            Self::new(window_id, TermMsgKind::Scroll(direction))
        }
        pub fn jump_to_bottom(window_id: Entity) -> Self {
            Self::new(window_id, TermMsgKind::JumpToBottom)
        }
        pub fn reflow(window_id: Entity) -> Self {
            Self::new(window_id, TermMsgKind::Reflow)
        }
        /// Writes a simple line to the buffer. For rich text support, see [Self::writeln_rich]
        pub fn write(term_id: Entity, line: impl ToString) -> Self {
            let line = line.to_string();
            Self::new(term_id, TermMsgKind::Write(vec![TermWrite::new(line)]))
        }
        /// Writes a rich line of text to the terminal. See [`TermWrite`] for
        /// more detail.
        pub fn write_spans(term_id: Entity, spans: Vec<TermWrite>) -> Self {
            Self::new(term_id, TermMsgKind::Write(spans))
        }
    }
}
pub use events::*;

pub mod helpers {
    use super::*;

    /// This struct hold all the necessary data to spawn a terminal text span in
    /// a convenient format. In order to facilitate text wrapping and ANSI
    /// support, [`VtLine`] data must contain the entire logical line, while the
    /// text spans must be spawned separately. This struct is designed to help
    /// with the API by making virtual text spans easier to author.
    #[derive(Debug, PartialEq, Reflect, Clone)]
    pub struct TermWrite {
        pub text: String,
        pub style: VtCellStyle,
    }
    impl TermWrite {
        pub fn new(text: impl ToString) -> Self {
            Self {
                text: text.to_string(),
                style: VtCellStyle::default(),
            }
        }
        pub fn with_color(self, color: impl Into<Color>) -> Self {
            Self {
                style: VtCellStyle {
                    color: color.into(),
                    ..self.style
                },
                ..self
            }
        }
        pub fn with_background(self, color: impl Into<Color>) -> Self {
            Self {
                style: VtCellStyle {
                    background: color.into(),
                    ..self.style
                },
                ..self
            }
        }
    }
}
pub use helpers::*;
