// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_dolly::prelude::*;

mod cam;
pub use cam::*;
use q_service::prelude::*;

#[derive(ServiceError, thiserror::Error, Debug, Clone, Copy, PartialEq)]
pub enum CamErr {}

#[derive(ServiceData, Debug, Clone, Copy, PartialEq, Default)]
pub struct CamData {
    pub can_scroll: bool,
}

service!(CamService, CamData, CamErr);

pub struct InspectorCamPlugin;
impl Plugin for InspectorCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_service(
            CamService::default_spec()
                .on_init(spawn_camera)
                .on_enable(set_cam_active::<true>)
                .on_disable(set_cam_active::<false>),
        )
        .add_systems(
            Update,
            (Dolly::<InspectorCam>::update_active, update_camera)
                .run_if(service_enabled(CamService::handle())),
        );
    }
}

pub(crate) fn spawn_camera(mut commands: Commands) -> Result<bool, CamErr> {
    info!("spawn_camera");
    let transform =
        Transform::from_xyz(2., 2., 5.).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn(InspectorCamBundle::new(transform));
    Ok(true)
}

fn set_cam_active<const VAL: bool>(
    mut cam: Single<&mut Camera, With<InspectorCam>>,
) -> Result<(), CamErr> {
    cam.is_active = VAL;
    Ok(())
}
