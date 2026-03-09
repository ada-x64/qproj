use bevy::ui::ui_layout_system;

use crate::prelude::*;

/// [`SystemSet`] for all terminal-related systems.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TerminalSystems {
    PreMutation,
    Mutation,
    PostMutation,
}

/// The primary plugin for q_term
#[derive(Default, Debug)]
pub struct TerminalPlugin;
impl Plugin for TerminalPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TermMsg>();
        app.add_message::<TermMsg>();
        app.add_systems(
            PostUpdate,
            (update_font, update_char_width, resize)
                .chain()
                .in_set(TerminalSystems::PreMutation),
        );
        app.add_systems(
            PostUpdate,
            (handle_messages, update_layout)
                .after(ui_layout_system)
                .chain()
                .in_set(TerminalSystems::Mutation),
        );
        app.configure_sets(
            PostUpdate,
            TerminalSystems::PreMutation.before(TerminalSystems::Mutation),
        );
        app.configure_sets(
            PostUpdate,
            TerminalSystems::Mutation.before(TerminalSystems::PostMutation),
        );
        app.configure_sets(
            PostUpdate,
            TerminalSystems::PostMutation.after(TerminalSystems::Mutation),
        );
    }
}
