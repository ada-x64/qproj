use bevy::ui::{self, ui_layout_system};

use crate::prelude::*;

/// [`SystemSet`] for all terminal-related systems.
#[derive(SystemSet, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TerminalSystems;

/// The primary plugin for q_term
#[derive(Default, Debug)]
pub struct TerminalPlugin;
impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TerminalMessage>();
        app.add_systems(
            PostUpdate,
            (handle_messages, update_layout)
                .chain()
                .in_set(TerminalSystems),
        );
    }
}
