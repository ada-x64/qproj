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
    #[derive(Component, Default, Deref, Debug)]
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
}
pub use components::*;

mod events {
    use super::*;
    // TODO: Since this is type-erased, we could accept either a terminal window or a terminal buffer id here.
    // That way we can write to buffers without necessarily having a window.
    #[derive(Debug, Message, Reflect, PartialEq, Eq, Hash)]
    pub struct TerminalMessage {
        /// Represents an entity containing a [`TerminalWindow`]. Any
        /// modifications aimed at the buffer will modify the related
        /// [`Terminal`]'s [`TerminalLine`] list.
        pub window_id: Entity,
        pub kind: TerminalMessageKind,
    }
    impl TerminalMessage {
        pub fn new(window_id: Entity, kind: TerminalMessageKind) -> Self {
            Self { window_id, kind }
        }
        pub fn reflow(window_id: Entity) -> Self {
            Self::new(window_id, TerminalMessageKind::Reflow)
        }
        pub fn writeln(window_id: Entity, line: impl ToString) -> Self {
            Self::new(window_id, TerminalMessageKind::Writeln(line.to_string()))
        }
        pub fn scroll(window_id: Entity, direction: isize) -> Self {
            Self::new(window_id, TerminalMessageKind::Scroll(direction))
        }
        pub fn jump_to_bottom(window_id: Entity) -> Self {
            Self::new(window_id, TerminalMessageKind::JumpToBottom)
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
    use bevy::{
        ecs::{lifecycle::HookContext, world::DeferredWorld},
        picking::hover::Hovered,
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
        Text,
        TextLayout {
            justify: Justify::Left,
            linebreak: LineBreak::NoWrap
        },
        TerminalScrollPos,
        TermWidth,
        TermHeight,
        TerminalLayout,
        Hovered,
    )]
    #[component(immutable, on_add=Self::on_add)]
    #[relationship(relationship_target = TerminalWindowList)]
    pub struct TerminalWindow(pub Entity);
    impl TerminalWindow {
        /// Spawn [`TerminalCharWidth`] et al
        pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
            let mut cmds = world.commands();
            let id = cmds
                .spawn((
                    TerminalCharWidth {
                        target: ctx.entity,
                        value: 0.,
                    },
                    Node::default(),
                    Visibility::Hidden,
                    Text::new(" "),
                ))
                .id();
            cmds.entity(ctx.entity)
                .add_one_related::<TerminalCharWidth>(id);
        }
    }

    /// Scroll position, in lines. 0 means you're at the bottom.
    #[derive(Component, Debug, Reflect, PartialEq, Eq, Hash, Clone, Copy, Deref, Default)]
    #[component(immutable)]
    pub struct TerminalScrollPos(pub usize);

    /// Width of a character cell in pixels, determined by measuring the width of a space.
    #[derive(Component, Debug, Reflect, PartialEq, Clone, Copy)]
    #[component(immutable)]
    #[relationship(relationship_target=TerminalCharWidthTarget)]
    pub struct TerminalCharWidth {
        #[relationship]
        pub target: Entity,
        pub value: f32,
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

    /// The number of columns in the terminal.
    #[derive(Component, Default, Deref, Debug)]
    #[component(immutable, on_insert=Self::on_insert)]
    pub struct TermWidth(pub usize);
    impl TermWidth {
        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            // Re-wrap the terminal text.
            if let Some(this) = world.get::<Self>(ctx.entity)
                && this.0 == 0
            {
                warn!("TermWidth set to 0");
            }
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
            if let Some(this) = world.get::<Self>(ctx.entity)
                && this.0 == 0
            {
                warn!("TermHeight set to 0");
            }
            world
                .commands()
                .write_message(TerminalMessage::reflow(ctx.entity));
        }
    }
}
pub use display::*;
