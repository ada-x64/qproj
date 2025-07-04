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

boolish_states!(PlayerCamState);

pub struct PlayerCamPlugin;
impl PlayerCamPlugin {
    fn set_cam_active<const VAL: bool>(
        mut cam: Single<&mut Camera, With<PlayerCam>>,
    ) {
        cam.is_active = VAL;
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
            .set_target_position(
                new_tf.translation,
                new_tf.rotation,
                lookat_pos,
            );
        **set.p2() = new_tf;
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
