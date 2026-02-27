use crate::prelude::*;

mod components {
    use super::*;
    use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};

    /// A simple marker component for the command prompt.
    /// Requires all the necessary components for the underlying implementation.
    /// Internally points to all the [`TerminalWindow`]s which display its content.
    #[derive(Component, Default, Reflect)]
    #[require(
        TermWidth,
        TermHeight,
        TerminalLines,
        TerminalLayout,
        TerminalWindowList
    )]
    pub struct Terminal;

    /// Tracks which [`TerminalWindow`] entities are displayed by this node's terminal.
    #[derive(Component, Default, Reflect, Deref, Debug)]
    #[relationship_target(relationship = TerminalWindow)]
    pub struct TerminalWindowList(Vec<Entity>);

    /// The number of columns in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TermWidth(pub usize);
    impl TermWidth {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            // Re-wrap the terminal text.
            world
                .commands()
                .write_message(TerminalMessage::reflow(ctx.entity));
        }
    }

    /// The number of rows in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TermHeight(pub usize);
    impl TermHeight {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            // Insert/remove LineRefs as appropriate.
            world
                .commands()
                .write_message(TerminalMessage::reflow(ctx.entity));
        }
    }

    /// This entity represents the underlying text buffer. It contains
    /// references to [`TerminalLine`] entities.
    #[derive(Component, Default, Deref, Debug)]
    #[relationship_target(relationship=TerminalLine)]
    pub struct TerminalLines(Vec<Entity>);

    /// A single, newline-delimited logical line.
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

    /// The terminal's layout grid. Contains references to [`TerminalRow`]
    /// entities. This entity represents the fully laid-out text buffer
    /// corresponding to the target terminal's [`TerminalLines`] entity.
    #[derive(Component, Default, Deref, Debug)]
    #[relationship_target(relationship=TerminalRow)]
    pub struct TerminalLayout(Vec<Entity>);

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
}
pub use components::*;

mod events {
    use super::*;
    #[derive(Debug, Message, Reflect, PartialEq, Eq, Hash)]
    pub struct TerminalMessage {
        pub target: Entity,
        pub kind: TerminalMessageKind,
    }
    impl TerminalMessage {
        pub fn new(target: Entity, kind: TerminalMessageKind) -> Self {
            Self { target, kind }
        }
        pub fn reflow(target: Entity) -> Self {
            Self::new(target, TerminalMessageKind::Reflow)
        }
        pub fn writeln(target: Entity, line: impl ToString) -> Self {
            Self::new(target, TerminalMessageKind::Writeln(line.to_string()))
        }
        pub fn scroll(target: Entity, direction: isize) -> Self {
            Self::new(target, TerminalMessageKind::Scroll(direction))
        }
        pub fn jump_to_bottom(target: Entity) -> Self {
            Self::new(target, TerminalMessageKind::JumpToBottom)
        }
    }
    #[derive(Debug, Reflect, PartialEq, Eq, Hash)]
    pub enum TerminalMessageKind {
        /// Reflow the terminal. Modifes LineRefs.
        Reflow,
        // TODO: Should be a TerminalLineBundle, i.e., should allow adding
        // textspan children. Perhaps this takes reference to an entity, so the
        // caller (e.g. the commands impl) has control over converting from raw
        // text to the final hierarchy.
        /// Write a line to the "buffer"
        Writeln(String),
        /// Scroll in the given direction. A scroll position of 0 means you are
        /// at the last line.
        Scroll(isize),
        /// Jump to the last line. Sets scroll value to 0.
        JumpToBottom,
    }
}
pub use events::*;

mod display {
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
        Text,
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::NoWrap
        },
        TerminalScrollPos,
        TerminalCharWidth,
    )]
    #[component(immutable)]
    #[relationship(relationship_target = TerminalWindowList)]
    pub struct TerminalWindow(pub Entity);

    /// Scroll position, in lines. 0 means you're at the bottom.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref, Default)]
    #[component(immutable)]
    pub struct TerminalScrollPos(pub usize);
}
pub use display::*;
