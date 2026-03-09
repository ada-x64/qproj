use crate::prelude::*;

mod terminfo {
    use super::*;
    use bevy::ecs::query::QueryData;

    /// Public API and query helpers for the [`Terminal`] entity.
    #[derive(QueryData, Debug)]
    pub struct TermInfo {
        pub id: Entity,
        pub line_discipline: &'static LineDiscipline,
        pub cursor: &'static TerminalCursor,
        pub(crate) lines: &'static TerminalLines,
        pub(crate) rows: &'static TerminalLayout,
        pub num_rows: &'static TermHeight,
        pub num_cols: &'static TermWidth,
        pub scroll_pos: &'static TerminalScrollPos,
    }
    impl<'w, 's> TermInfoItem<'w, 's> {
        #[inline(always)]
        pub fn grid_lines<'a>(
            &self,
            q_lines: &'a Query<(Entity, &TerminalLine)>,
        ) -> impl Iterator<Item = (Entity, &'a TerminalLine)> {
            q_lines.iter_many(self.lines.iter())
        }

        #[inline(always)]
        pub fn grid<'a>(
            &self,
            q_lines: &'a Query<(Entity, &TerminalLine)>,
            q_spans: &'a Query<(Entity, &VirtualTextSpan)>,
        ) -> impl Iterator<Item = (Entity, &'a TerminalLine, Vec<(Entity, &'a VirtualTextSpan)>)>
        {
            q_lines.iter_many(self.lines.iter()).map(|(line_id, line)| {
                let spans = q_spans
                    .iter()
                    .filter(|(_, span)| span.line == line_id)
                    .collect::<Vec<_>>();
                (line_id, line, spans)
            })
        }

        #[inline(always)]
        pub fn viewport_rows<'a>(
            &self,
            q_lines: &'a Query<&TerminalRow>,
        ) -> impl Iterator<Item = &'a TerminalRow> {
            q_lines.iter_many(self.rows.iter())
        }

        #[inline(always)]
        pub fn virtual_spans<'a>(
            &self,
            q_lines: &'a Query<&Children, With<TerminalLine>>,
            q_spans: &'a Query<&VirtualTextSpan>,
        ) -> impl Iterator<Item = &'a VirtualTextSpan> {
            q_lines
                .iter_many(self.lines.iter())
                .flat_map(|children| q_spans.iter_many(children))
        }

        pub fn mutate(&self, commands: &mut Commands, msg: TermMsgKind) {
            commands.write_message(TermMsg::new(self.id, msg));
        }
    }
}
pub use terminfo::*;

mod components {

    use super::*;

    /// Marker component / entry point for spawning the underlying terminal
    /// buffer.
    #[derive(Component, Default, Reflect)]
    #[require(TerminalLines, TerminalWindow, LineDiscipline, TerminalCursor)]
    pub struct Terminal;

    /// Cursor for the [`Terminal`]. Points at a given byte index into a
    /// [`TerminalLine`] (nth from end).
    #[derive(Component, Default, Reflect, Clone, Copy, Debug)]
    pub struct TerminalCursor {
        pub line: usize,
        pub char: usize,
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

    // TODO: Add a limit to the number of stored lines.
    // This can't currently be a vecdeque due to trait constraints,
    // so need to add a manual 'maximum' field and do checks on insert
    // to ensure the vec doesn't overload its capacity.
    /// This entity represents the underlying text buffer. It contains
    /// references to [`TerminalLine`] entities.
    #[derive(Component, Default, Deref, Debug, Reflect)]
    #[relationship_target(relationship=TerminalLine)]
    pub struct TerminalLines(Vec<Entity>);

    /// A single, newline-delimited logical line. Does _not_ include the
    /// trailing newline.
    #[derive(Component, Deref, DerefMut, Debug, Reflect, PartialEq, Eq, Hash, Clone)]
    #[component(immutable)]
    #[relationship(relationship_target=TerminalLines)]
    pub struct TerminalLine {
        #[deref]
        pub value: String,
        #[relationship]
        target: Entity,
    }
    impl TerminalLine {
        pub fn new(terminal_id: Entity, value: String) -> Self {
            Self {
                value,
                target: terminal_id,
            }
        }
    }

    /// Marker component for an entity which acts as a virtual [`TextSpan`] for
    /// the underlying [`TerminalLine`]. This component's range will be used to create
    /// [`TextSpan`] child entities for the [`TerminalRow`] container entities.
    ///
    /// ### Can I set the font style?
    /// Bold, italic, underlined, etc. would
    /// require setting the font ID without modifying the font size. We _could_
    /// allow users to pass a font ID, but there is no guarantee that the user
    /// would occupy that slot with a true font variant. In order to assure
    /// safety, we would need to register font variants _before_ allowing the
    /// user to set them, which is outside the scope of this crate.
    #[derive(Component, Debug, Reflect, PartialEq, Clone, Copy)]
    pub struct VirtualTextSpan {
        /// reference to the line this span is part of
        pub line: Entity,
        /// offset in characters
        pub offset: usize,
        /// character length of the span
        pub range: usize,
        pub style: VspanStyle,
    }

    /// Text styles for [`VirtualTextSpanSpawner`]s.
    #[derive(Clone, Copy, Debug, PartialEq, Reflect)]
    pub struct VspanStyle {
        pub color: Color,
        pub background: Color,
    }
    impl Default for VspanStyle {
        fn default() -> Self {
            Self {
                color: Color::WHITE,
                background: Color::BLACK,
            }
        }
    }

    /// This struct hold all the necessary data to spawn a terminal text span in
    /// a convenient format. In order to facilitate text wrapping,
    /// [`TerminalLine`] data must contain the entire string, while the text
    /// spans must be separate. This struct is designed to help with the API
    /// by making virtual text spans easier to author.
    #[derive(Debug, PartialEq, Reflect, Clone)]
    pub struct VirtualTextSpanSpawner {
        pub text: String,
        pub style: VspanStyle,
    }
    impl VirtualTextSpanSpawner {
        pub fn new(text: impl ToString) -> Self {
            Self {
                text: text.to_string(),
                style: VspanStyle::default(),
            }
        }
        pub fn with_color(self, color: impl Into<Color>) -> Self {
            Self {
                style: VspanStyle {
                    color: color.into(),
                    ..self.style
                },
                ..self
            }
        }
        pub fn with_background(self, color: impl Into<Color>) -> Self {
            Self {
                style: VspanStyle {
                    background: color.into(),
                    ..self.style
                },
                ..self
            }
        }
    }

    /// Convenience macro for creating vec of [`VirtualTextSpanSpawner`]s.
    ///
    /// ### Example
    /// ```rust
    /// # use bevy::color::palettes::css;
    /// let msg = term_writeln!(
    ///     "This is some ",
    ///     ("fancy", background = css::RED, color = css::WHITE),
    ///     (" text!", color = css::BLACK)
    /// );
    /// let expected = vec![
    ///     VirtualTextSpanSpawner::new("This is some "),
    ///     VirtualTextSpanSpawner::new("fancy")
    ///         .with_background_color(css::RED)
    ///         .with_color(css::WHITE),
    ///     VirtualTextSpanSpawner::new(" text!").with_color(css::BLACK),
    /// ];
    /// assert_eq!(msg, expected);
    /// ```
    #[macro_export]
    macro_rules! term_writeln {
        ($($value:tt),*) => {
             vec![$(term_writeln!(@expr_parse $value)),*]
        };
        (@expr_parse $string:literal) => {
            VirtualTextSpanSpawner::new($string)
        };
        (@expr_parse $string:ident) => {
            VirtualTextSpanSpawner::new($string)
        };
        (@expr_parse ($string:literal $(, background=$bg:expr)? $(, color=$color:expr)?)) => {
            term_writeln!(@rich_expr $string, $($bg)?, $($color)?)
        };
        (@expr_parse ($string:literal $(, color=$color:expr)?) $(, background=$bg:expr)? ) => {
            term_writeln!(@rich_expr $string, $($bg)?, $($color)?)
        };
        (@expr_parse ($string:ident $(, background=$bg:expr)? $(, color=$color:expr)?)) => {
            term_writeln!(@rich_expr $string, $($bg)?, $($color)?)
        };
        (@expr_parse ($string:ident $(, color=$color:expr)?) $(, background=$bg:expr)? ) => {
            term_writeln!(@rich_expr $string, $($bg)?, $($color)?)
        };
        (@rich_expr $string:literal, $($bg:expr)?, $($color:expr)?) => {
            VirtualTextSpanSpawner::new($string)$(.with_color($color))?$(.with_background($bg))?
        };
        (@rich_expr $string:ident, $($bg:expr)?, $($color:expr)?) => {
            VirtualTextSpanSpawner::new($string)$(.with_color($color))?$(.with_background($bg))?
        };
    }
}
pub use components::*;

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
        Write(Vec<VirtualTextSpanSpawner>),
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
            Self::new(term_id, TermMsgKind::Write(term_writeln!(line)))
        }
        /// Writes a rich line of text to the terminal See [`VirtualTextSpan`],
        /// [`term_writeln!`], and [`VirtualTextSpanSpawner`] for more
        /// information.
        pub fn write_spans(term_id: Entity, spans: Vec<VirtualTextSpanSpawner>) -> Self {
            Self::new(term_id, TermMsgKind::Write(spans))
        }
    }
}
pub use events::*;

mod display {
    use bevy::{
        ecs::{lifecycle::HookContext, world::DeferredWorld},
        picking::hover::DirectlyHovered,
        text::LineHeight,
    };

    use super::*;

    /// A terminal display. Will spawn a new [`Node`] sized to the parent
    /// container and populate the hierarchy with [`TextSpan`] components according
    /// to the target entity's properties.
    #[derive(Component, Reflect, Default)]
    #[require(
        Node {
            overflow: Overflow::clip(),
            ..Default::default()
        },
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::NoWrap
        },
        TerminalScrollPos,
        TermWidth,
        TermHeight,
        TerminalLayout,
        Pickable,
        // Requires text metadata to propogate to its children
        TextColor,
        TextFont,
        LineHeight,
    )]
    #[component(immutable, on_add=Self::on_add)]
    pub struct TerminalWindow;
    impl TerminalWindow {
        /// Spawn [`TerminalCharWidth`] et al
        pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            let mut commands = world.commands();
            let id = commands.spawn(TerminalCharWidth::new(ctx.entity, 0.)).id();
            commands
                .entity(ctx.entity)
                .add_one_related::<TerminalCharWidth>(id)
                .observe(scroll_terminal_window)
                .observe(|event: On<Pointer<Over>>| {
                    debug!(?event);
                });
        }
    }

    /// Scroll position, in lines. 0 means you're at the bottom.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref, Default)]
    #[component(immutable)]
    pub struct TerminalScrollPos(pub usize);

    /// Width of a character cell in pixels, determined by measuring the width of a space.
    #[derive(Component, Debug, Reflect, PartialEq, Clone, Copy)]
    #[component(immutable)]
    #[require(Node::default(), Pickable::IGNORE, Visibility::Hidden, Text::new(" "))]
    #[relationship(relationship_target=TerminalCharWidthTarget)]
    pub struct TerminalCharWidth {
        #[relationship]
        target: Entity,
        value: f32,
    }
    impl TerminalCharWidth {
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

    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref)]
    #[relationship_target(relationship=TerminalCharWidth, linked_spawn)]
    pub struct TerminalCharWidthTarget(Entity);
    impl TerminalCharWidthTarget {
        pub fn target(&self) -> Entity {
            self.0
        }
    }

    /// The terminal's layout grid. Contains references to [`TerminalRow`]
    /// entities. This entity represents the fully laid-out text buffer
    /// corresponding to the target terminal's [`TerminalLines`] entity.
    #[derive(Component, Default, Deref, Debug, Reflect)]
    #[relationship_target(relationship=TerminalRow)]
    #[component(on_add = Self::on_add)]
    pub struct TerminalLayout(Vec<Entity>);
    impl TerminalLayout {
        fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            let child_node = world.commands().spawn(TerminalTextWrapper).id();
            world.commands().entity(ctx.entity).add_child(child_node);
        }
    }
    /// A marker component for the [`TerminalLayout`] entity's child component,
    /// which contains the automatically created [`TextSpan`] children;
    #[derive(Component, Default, Debug, Reflect)]
    #[require(Node, Text, DirectlyHovered, Pickable)]
    pub struct TerminalTextWrapper;

    /// A reference to a logical line with a character offset into its full text value.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy)]
    #[relationship(relationship_target=TerminalLayout)]
    #[component(immutable)]
    pub struct TerminalRow {
        /// Reference to an entity containing a [`TerminalLine`].
        /// If the line is empty, this will be None.
        pub line: Option<Entity>,
        /// Character offset into the line at which to begin this span.
        pub offset: usize,
        /// Relationship target
        #[relationship]
        term_id: Entity,
    }
    impl TerminalRow {
        pub fn empty(term_id: Entity) -> Self {
            Self {
                line: None,
                offset: 0,
                term_id,
            }
        }
        pub fn new(term_id: Entity, line: Entity, offset: usize) -> Self {
            Self {
                line: Some(line),
                offset,
                term_id,
            }
        }
    }

    /// The number of columns in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TermWidth(pub usize);
    impl TermWidth {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            world.commands().write_message(TermMsg::reflow(ctx.entity));
        }
    }

    /// The number of rows in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TermHeight(pub usize);
    impl TermHeight {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            world.commands().write_message(TermMsg::reflow(ctx.entity));
        }
    }
}
pub use display::*;
