// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_dolly::prelude::*;

mod cam;
use crate::state::Services;
pub use cam::*;
use q_service::prelude::*;

#[derive(ServiceError, thiserror::Error, Debug, Clone, Copy, PartialEq)]
pub enum InspectorCamErr {}

#[derive(ServiceData, Debug, Clone, Copy, PartialEq, Default)]
pub struct InspectorCamData {
    can_scroll: bool,
}

service!(InspectorCam, Services, InspectorCamData, InspectorCamErr);

pub struct InspectorCamPlugin;
impl Plugin for InspectorCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_service(
            InspectorCamServiceSpec::new(Services::InspectorCam)
                .on_init(|world| {
                    use bevy::ecs::system::RunSystemOnce;
                    world.run_system_once(spawn_camera);
                    Ok(true)
                })
                .on_enable(|world| {
                    world.run_system_cached(set_cam_active::<true>)
                })
                .on_disable(|world| {
                    let id =
                        world.register_system_cached(set_cam_active::<false>);
                    world.run_system_cached(id);
                    Ok(())
                }),
        )
        .add_systems(
            Update,
            (Dolly::<InspectorCam>::update_active, update_camera)
                .run_if(service_enabled(INSPECTOR_CAM_SERVICE)),
        );
    }
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
