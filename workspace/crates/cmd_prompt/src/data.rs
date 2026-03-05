use crate::prelude::*;

mod components {
    use super::*;

    // TODO: Add a limit to the number of stored lines.
    /// Marker component / entry point for spawning the underlying terminal
    /// buffer.
    #[derive(Component, Default, Reflect)]
    #[require(TerminalLines, TerminalWindowList)]
    pub struct Terminal;

    /// Tracks which [`TerminalWindow`] entities are displayed by this node's
    /// terminal.
    #[derive(Component, Default, Reflect, Deref, Debug)]
    #[relationship_target(relationship = TerminalWindow)]
    pub struct TerminalWindowList(Vec<Entity>);

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
    #[derive(Component, Debug, Reflect, PartialEq)]
    pub struct VirtualTextSpan {
        pub start: usize,
        pub end: usize,
        pub color: Option<Color>,
        pub background_color: Option<Color>,
    }
    impl VirtualTextSpan {
        pub fn spawn_textspan(self, text: &str, commands: &mut Commands) -> Entity {
            let color = self.color.map(TextColor).unwrap_or_default();
            let bg = self
                .background_color
                .map(TextBackgroundColor)
                .unwrap_or_default();
            commands
                .spawn((TextSpan(text[self.start..self.end].to_string()), color, bg))
                .id()
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
        pub color: Option<Color>,
        pub background_color: Option<Color>,
    }
    impl VirtualTextSpanSpawner {
        pub fn new(text: impl ToString) -> Self {
            Self {
                text: text.to_string(),
                color: None,
                background_color: None,
            }
        }
        pub fn with_color(self, color: impl Into<Color>) -> Self {
            Self {
                color: Some(color.into()),
                ..self
            }
        }
        pub fn with_background_color(self, color: impl Into<Color>) -> Self {
            Self {
                background_color: Some(color.into()),
                ..self
            }
        }
        /// Adds a [`TerminalLine`] and the corresponding [`VirtualTextSpans`] to the given target entity.
        /// Returns the [`Entity`] ID of the spawned [`TerminalLine`] and its corresponding string length.
        pub(crate) fn spawn(
            spans: &[VirtualTextSpanSpawner],
            target: Entity,
            commands: &mut Commands,
        ) -> (Entity, usize) {
            let (text, children) =
                spans
                    .iter()
                    .fold((String::new(), vec![]), |(text, ids), span| {
                        let start = text.len();
                        let end = start + span.text.len();
                        let child = commands
                            .spawn(VirtualTextSpan {
                                start,
                                end,
                                color: span.color,
                                background_color: span.background_color,
                            })
                            .id();
                        (
                            text + &span.text,
                            [ids, vec![child]].into_iter().flatten().collect::<Vec<_>>(),
                        )
                    });
            let len = text.len();
            let line_id = commands
                .spawn(TerminalLine::new(target, text))
                .add_children(&children)
                .id();
            commands
                .entity(target)
                .add_one_related::<TerminalLine>(line_id);
            (line_id, len)
        }
    }

    /// Convenience macro for creating vec of [`VirtualTextSpanSpawner`]s.
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
            VirtualTextSpanSpawner::new($string)$(.with_color($color))?$(.with_background_color($bg))?
        };
        (@rich_expr $string:ident, $($bg:expr)?, $($color:expr)?) => {
            VirtualTextSpanSpawner::new($string)$(.with_color($color))?$(.with_background_color($bg))?
        };
    }
    #[test]
    fn test_writeln_macro() {
        use bevy::color::palettes::css;
        let msg = term_writeln!(
            "This is some ",
            ("fancy", background = css::RED, color = css::WHITE),
            (" text!", color = css::BLACK)
        );
        let expected = vec![
            VirtualTextSpanSpawner::new("This is some "),
            VirtualTextSpanSpawner::new("fancy")
                .with_background_color(css::RED)
                .with_color(css::WHITE),
            VirtualTextSpanSpawner::new(" text!").with_color(css::BLACK),
        ];
        assert_eq!(msg, expected);
    }
}
pub use components::*;

mod events {
    use super::*;
    #[derive(Debug, Message, Reflect)]
    pub struct TerminalMessage {
        /// Represents an entity containing a [`Terminal`]. Any modifications
        /// aimed at the buffer will modify the related [`Terminal`]'s
        /// [`TerminalLine`] list.
        pub target: Entity,
        pub kind: TerminalMessageKind,
    }
    impl TerminalMessage {
        pub fn new(window_id: Entity, kind: TerminalMessageKind) -> Self {
            Self {
                target: window_id,
                kind,
            }
        }
        /// Writes a simple line to the buffer. For rich text support, see [Self::writeln_rich]
        pub fn writeln(window_id: Entity, line: impl ToString) -> Self {
            let line = line.to_string();
            Self::new(window_id, TerminalMessageKind::Writeln(term_writeln!(line)))
        }
        /// Writes a rich line of text to the terminal See [`VirtualTextSpan`],
        /// [`term_writeln!`], and [`VirtualTextSpanSpawner`] for more
        /// information.
        pub fn writeln_rich(window_id: Entity, spans: Vec<VirtualTextSpanSpawner>) -> Self {
            Self::new(window_id, TerminalMessageKind::Writeln(spans))
        }
    }
    #[derive(Debug, Reflect, Clone)]
    pub enum TerminalMessageKind {
        /// Write a line to the "buffer"
        Writeln(Vec<VirtualTextSpanSpawner>),
    }

    #[derive(Debug, Message, Reflect)]
    pub struct TerminalWindowMessage {
        pub target: Entity,
        pub kind: TerminalWindowMsgKind,
    }
    #[derive(Debug, Reflect)]
    pub enum TerminalWindowMsgKind {
        /// Reflow the terminal. Modifes LineRefs.
        Reflow,
        /// Scroll in the given direction. A scroll position of 0 means you are
        /// at the last line.
        Scroll(isize),
        /// Jump to the last line. Sets scroll value to 0.
        JumpToBottom,
        /// Modifications to the underlying terminal
        Terminal(TerminalMessageKind),
    }
    impl TerminalWindowMessage {
        pub fn new(window_id: Entity, kind: TerminalWindowMsgKind) -> Self {
            Self {
                target: window_id,
                kind,
            }
        }
        pub fn scroll(window_id: Entity, direction: isize) -> Self {
            Self::new(window_id, TerminalWindowMsgKind::Scroll(direction))
        }
        pub fn jump_to_bottom(window_id: Entity) -> Self {
            Self::new(window_id, TerminalWindowMsgKind::JumpToBottom)
        }
        pub fn reflow(window_id: Entity) -> Self {
            Self::new(window_id, TerminalWindowMsgKind::Reflow)
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
    #[derive(Component, Reflect)]
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
    #[relationship(relationship_target = TerminalWindowList)]
    pub struct TerminalWindow(pub Entity);
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
            world
                .commands()
                .write_message(TerminalWindowMessage::reflow(ctx.entity));
        }
    }

    /// The number of rows in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TermHeight(pub usize);
    impl TermHeight {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            world
                .commands()
                .write_message(TerminalWindowMessage::reflow(ctx.entity));
        }
    }
}
pub use display::*;
