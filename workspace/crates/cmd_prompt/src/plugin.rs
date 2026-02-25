use crate::prelude::*;

pub struct CommandPromptPlugin;
impl Plugin for CommandPromptPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TerminalMessage>();
        app.add_systems(PostUpdate, handle_messages);
    }
}
