// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod bundle;
mod driver;
pub use bundle::*;
pub use driver::*;
use q_service::prelude::*;
use tiny_bail::prelude::*;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_dolly::prelude::*;

#[derive(Debug, Clone, thiserror::Error)]
pub enum PlayerCamError {
    #[error("No player camera")]
    NoCam,
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

pub struct PlayerCamPlugin;
impl Plugin for PlayerCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_service(
            SimpleServiceSpec::<PlayerCamError>::new("PlayerCam".into())
                .is_startup(true)
                .on_enable(set_cam_active::<true>)
                .on_disable(set_cam_active::<true>),
        );
        app.add_systems(
            Update,
            (Dolly::<PlayerCam>::update_active, update_camera).run_if(
                |services: Query<&SimpleService<PlayerError>>| {
                    let s = r!(services.iter().find(|s| &s.name == "Player"));
                    matches!(s.state, ServiceState::Enabled)
                },
            ),
        );
    }
}

// pub const CAM_SERVICE_SPEC: ServiceSpec<PlayerServices> = ServiceSpec::<_> {
//     name: PlayerServices::Cam,
//     deps: vec![],
//     is_startup: true,
//     hooks: ServiceHooks {
//         on_enable: set_cam_active::<true>,
//         on_disable: set_cam_active::<false>,
//         on_init: default_init,
//         on_failure: default_fail,
//     },
// };

// enable/disable hook
fn set_cam_active<const VAL: bool>(
    world: &mut World,
) -> Result<(), PlayerCamError> {
    let mut cam = r!(
        Err(PlayerCamError::NoCam),
        world
            .query_filtered::<&mut Camera, With<PlayerCam>>()
            .single_mut(world)
    );
    cam.is_active = VAL;
    Ok(())
}
