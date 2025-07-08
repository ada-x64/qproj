// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_dolly::prelude::*;
use q_utils::InspectorIgnore;

#[derive(Component, Debug, Default)]
pub struct InspectorCam;

pub struct InspectorCamBundle;
impl InspectorCamBundle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(transform: Transform) -> impl Bundle {
        (
            Camera3d::default(),
            transform,
            Name::new("Inspector Cam"),
            InspectorCam,
            InspectorIgnore,
            Rig::builder()
                .with(Fpv::from_position_target(transform))
                .build(),
        )
    }
}
