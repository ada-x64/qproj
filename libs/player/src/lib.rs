// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;

pub mod cam;
pub mod controls;
pub mod player;
mod services;

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
