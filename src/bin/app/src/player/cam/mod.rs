// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

mod bundle;
mod driver;
pub use bundle::*;
pub use driver::*;
use q_service::prelude::*;

use crate::player::prelude::*;
use bevy::prelude::*;
use bevy_dolly::prelude::*;

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum CamError {
    #[error("No player camera")]
    NoCam,
}

#[derive(Default, Debug, Resource)]
pub struct CamService;
impl Service for CamService {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            .is_startup(true)
            .on_up(|mut cam: Single<&mut Camera, With<PlayerCam>>| {
                cam.is_active = true;
                Ok(())
            })
            .on_down(
                |_: In<DownReason>, mut cam: Single<&mut Camera, With<PlayerCam>>| {
                    cam.is_active = false;
                },
            );
        scope.add_systems(
            Update,
            (Dolly::<PlayerCam>::update_active, update_camera)
                .run_if(service_up::<PlayerService>()),
        );
    }
}

pub struct PlayerCamPlugin;
impl Plugin for PlayerCamPlugin {
    fn build(&self, app: &mut App) {
        app.register_service::<CamService>();
    }
}

pub fn update_camera(
    mut set: ParamSet<(
        Single<&Transform, With<Player>>,
        Single<&mut Rig, With<PlayerCam>>,
        Single<&mut Transform, With<PlayerCam>>,
    )>,
) {
    let mut new_tf = **set.p0();
    let lookat_pos = set.p0().translation;
    new_tf.translation -= Vec3::new(0., -2., -10.);
    new_tf.rotate_around(set.p0().translation, set.p0().rotation);
    new_tf.look_at(set.p0().translation, Vec3::Y);
    set.p1()
        .driver_mut::<PlayerCamDriver>()
        .set_target_position(new_tf.translation, new_tf.rotation, lookat_pos);
    **set.p2() = new_tf;
}
