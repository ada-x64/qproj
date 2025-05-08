use bevy::prelude::*;
use bevy_dolly::prelude::*;

#[derive(Component, Debug, Default)]
pub struct InspectorCam;

pub struct InspectorCamBundle;
impl InspectorCamBundle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(transform: Transform) -> impl Bundle {
        (
            Camera3d::default(),
            transform,
            InspectorCam,
            Rig::builder()
                .with(Fpv::from_position_target(transform))
                .build(),
        )
    }
}
