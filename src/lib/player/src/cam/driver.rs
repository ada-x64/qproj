// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_dolly::prelude::*;
use derive_builder::Builder;

#[derive(Builder)]
pub struct PlayerCamRigOptions {
    #[builder(default = "Vec3::ZERO")]
    position: Vec3,
    #[builder(default = "1.25")]
    rot_smoothing: f32,
    #[builder(default = "Vec3::new(0.0, 1.5, -3.5)")]
    arm_pos: Vec3,
    #[builder(default = "2.5")]
    arm_smoothing: f32,
    #[builder(default = "1.25")]
    lookat_smoothing: f32,
}

#[derive(Component, Deref, DerefMut, Debug)]
pub struct PlayerCamDriver(pub CameraRig);

impl PlayerCamDriver {
    pub fn new(opts: PlayerCamRigOptions) -> Self {
        Self(
            CameraRig::builder()
                .with(Position::new(opts.position))
                .with(Rotation::new(Quat::IDENTITY))
                .with(Smooth::new_position(opts.rot_smoothing).predictive(true))
                .with(Arm::new(opts.arm_pos))
                .with(Smooth::new_position(opts.arm_smoothing))
                .with(
                    LookAt::new(opts.position + Vec3::Y)
                        .tracking_smoothness(opts.lookat_smoothing)
                        .tracking_predictive(true),
                )
                .build(),
        )
    }
    pub fn set_target_position(
        &mut self,
        target_position: Vec3,
        target_rotation: Quat,
        lookat_target: Vec3,
    ) {
        self.driver_mut::<Position>().position = target_position; // - self.driver_mut::<Arm>().offset;
        self.driver_mut::<Rotation>().rotation = target_rotation;
        self.driver_mut::<LookAt>().target = lookat_target;
    }
}
impl RigDriver for PlayerCamDriver {
    fn update(&mut self, params: RigUpdateParams) -> Transform {
        self.0.update(params.delta_time_seconds)
    }
}
