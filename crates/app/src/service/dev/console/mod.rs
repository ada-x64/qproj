mod commands;
mod ui;

use crate::prelude::*;

pub mod prelude {
    pub use super::commands::*;
    pub use super::ui::*;
}

pub fn plugin(app: &mut App) {
    app.add_plugins(q_cmd_prompt::ConsolePlugin);
    app.add_plugins((ui::plugin, commands::plugin));
}
