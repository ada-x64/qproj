mod bundle;
mod driver;
pub use bundle::*;
pub use driver::*;
use q_utils::boolish_states;

use crate::prelude::*;
use bevy::prelude::*;
use bevy_dolly::prelude::*;
use std::f32::consts::PI;

boolish_states!(PlayerCamState);

pub struct PlayerCamPlugin;
impl PlayerCamPlugin {
    fn set_cam_active<const VAL: bool>(
        mut cam: Single<&mut Camera, With<PlayerCam>>,
    ) {
        cam.is_active = VAL;
    }

    #[allow(clippy::type_complexity)]
    pub fn update_camera(
        player_tf: Single<&Transform, With<Player>>,
        mut rig_tf: Single<&mut Rig, With<PlayerCam>>,
    ) {
        rig_tf.driver_mut::<PlayerCamDriver>().set_position(
            player_tf.translation - Vec3::new(0., -1., -1.),
            Quat::from_axis_angle(player_tf.forward().as_vec3(), PI / 3.),
        );
    }
}
impl Plugin for PlayerCamPlugin {
    fn build(&self, app: &mut App) {
        app.setup_boolish_states()
            .add_systems(
                OnEnter(PlayerCamState::Enabled),
                Self::set_cam_active::<true>,
            )
            .add_systems(
                OnEnter(PlayerCamState::Disabled),
                Self::set_cam_active::<false>,
            )
            .add_systems(
                Update,
                (Dolly::<PlayerCam>::update_active, Self::update_camera)
                    .run_if(in_state(PlayerState::Enabled)),
            );
    }
}
