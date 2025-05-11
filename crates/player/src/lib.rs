//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;

mod cam;
mod controls;
mod player;

pub mod prelude {
    pub use crate::cam::*;
    pub use crate::controls::*;
    pub use crate::player::*;
}
use prelude::*;
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((IntegrationPlugin, PlayerCamPlugin, ControlsPlugin));
    }
}
