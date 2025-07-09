// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_dolly::prelude::*;

mod cam;
pub use cam::*;
use q_utils::service;

service!(InspectorCam);
service!(InspectorCamScroll);

pub struct InspectorCamPlugin;
impl Plugin for InspectorCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            InspectorCamServicePlugin,
            InspectorCamScrollServicePlugin,
        ))
        .add_systems(
            OnEnter(InspectorCamStates::Initializing),
            (spawn_camera, trigger_completion).chain(),
        )
        .add_systems(
            Update,
            (Dolly::<InspectorCam>::update_active, update_camera)
                .run_if(in_state(InspectorCamStates::Enabled)),
        )
        .add_systems(
            OnEnter(InspectorCamStates::Disabled),
            set_cam_active::<false>,
        )
        .add_systems(
            OnEnter(InspectorCamStates::Enabled),
            set_cam_active::<true>,
        );
    }
}

fn trigger_completion(mut commands: Commands) {
    commands.trigger(InspectorCamInitialized(Ok(true)));
}
pub(crate) fn spawn_camera(mut commands: Commands) {
    let transform =
        Transform::from_xyz(2., 2., 5.).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn(InspectorCamBundle::new(transform));
}

fn set_cam_active<const VAL: bool>(
    mut cam: Single<&mut Camera, With<InspectorCam>>,
) {
    cam.is_active = VAL;
}
