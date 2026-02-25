use crate::prelude::*;

mod components {
    use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};

    use super::*;
    /// A simple marker component for the command prompt.
    #[derive(Component, Default)]
    #[require(
        TerminalCols,
        TerminalRows,
        TerminalLines,
        TerminalLayout,
        TerminalScrollPos
    )]
    #[component(on_add=Self::on_add)]
    pub struct Terminal;
    impl Terminal {
        fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            world
                .commands()
                .write_message(TerminalMessage::reflow(ctx.entity));
        }
    }

    /// The number of columns in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TerminalCols(pub usize);
    impl TerminalCols {
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
    pub struct TerminalRows(pub usize);
    impl TerminalRows {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            // Insert/remove LineRefs as appropriate.
            world
                .commands()
                .write_message(TerminalMessage::reflow(ctx.entity));
        }
    }

    /// Relationship target for [`TerminalLine`]s.
    #[derive(Component, Default, Deref, Debug)]
    #[relationship_target(relationship=TerminalLine, linked_spawn)]
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
        pub fn new(value: String, relationship_target: Entity) -> Self {
            Self {
                value,
                target: relationship_target,
            }
        }
    }

    /// Relationship target for [`LineRef`]s.
    #[derive(Component, Default, Deref, Debug)]
    #[relationship_target(relationship=TerminalRow, linked_spawn)]
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
        parent: Entity,
    }
    impl TerminalRow {
        pub fn empty(parent: Entity) -> Self {
            Self {
                line: None,
                offset: 0,
                parent,
            }
        }
        pub fn new(line: Entity, offset: usize, parent: Entity) -> Self {
            Self {
                line: Some(line),
                offset,
                parent,
            }
        }
    }

    /// Scroll position, in lines. 0 means you're at the bottom.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref, Default)]
    #[component(immutable, on_insert = Self::on_insert)]
    pub struct TerminalScrollPos(pub usize);
    impl TerminalScrollPos {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            world
                .commands()
                .write_message(TerminalMessage::reflow(ctx.entity));
        }
    }
}
pub use components::*;

pub mod events {
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
    impl TerminalMessageKind {
        /// Priorty in which messages should be handled. Typically buffer
        /// changes go first, then visual changes are applied.
        /// Lower priority means it goes first.
        pub fn priority(&self) -> usize {
            match self {
                TerminalMessageKind::Writeln(_) => 0,
                TerminalMessageKind::Scroll(_) | TerminalMessageKind::JumpToBottom => 90,
                TerminalMessageKind::Reflow => 100,
            }
        }
    }
    impl PartialOrd for TerminalMessageKind {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for TerminalMessageKind {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.priority().cmp(&other.priority())
        }
    }
}
pub use events::*;
