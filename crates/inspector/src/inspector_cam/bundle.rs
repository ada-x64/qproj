//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
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
