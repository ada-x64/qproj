// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

use bevy::prelude::*;
use q_service::prelude::*;

pub mod bundle;
pub mod cam;
pub mod controls;
pub mod service;

pub mod prelude {
    pub use super::bundle::*;
    pub use super::cam::*;
    pub use super::controls::*;
    pub use super::service::*;
}
use prelude::*;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_service::<PlayerService>();
        app.add_plugins((PlayerCamPlugin, ControlsPlugin));
    }
}
