use crate::prelude::*;

mod clear;
mod echo;
mod set;
mod show;

pub mod prelude {
    pub use super::clear::clear_buffer;
}

pub fn plugin(app: &mut App) {
    app.add_plugins((show::plugin, echo::plugin, clear::plugin, set::plugin));
}
