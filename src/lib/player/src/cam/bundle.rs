// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_dolly::prelude::*;

use super::{PlayerCamDriver, PlayerCamRigOptionsBuilder};

#[derive(Component, Default, Debug)]
pub struct PlayerCam;

pub struct PlayerCamBundle;
impl PlayerCamBundle {
    /// TODO: Make this a struct instead of a big tuple?
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> impl Bundle {
        (
            Name::new("Player Cam"),
            Camera3d::default(),
            PlayerCam,
            Rig::builder()
                .with(PlayerCamDriver::new(
                    PlayerCamRigOptionsBuilder::default().build().unwrap(),
                ))
                .build(),
            Transform::default(),
            PointLight::default(),
        )
    }
}
