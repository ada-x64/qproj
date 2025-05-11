//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
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
        mut set: ParamSet<(
            Single<&Transform, With<Player>>,
            Single<&mut Rig, With<PlayerCam>>,
            Single<&mut Transform, With<PlayerCam>>,
        )>,
    ) {
        let cam_dist = 10.;
        let translation =
            set.p0().translation - Vec3::new(0., cam_dist, cam_dist);
        let rotation =
            Quat::from_axis_angle(set.p0().forward().as_vec3(), PI / 3.);
        set.p1()
            .driver_mut::<PlayerCamDriver>()
            .set_target_position(translation, rotation);
        set.p2().translation = translation;
        set.p2().rotation = rotation;
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
