// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

use bevy::prelude::*;
use bevy_dolly::prelude::*;

mod cam;
pub use cam::*;
use q_service::prelude::*;

#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct CamService {
    pub can_scroll: bool,
}
impl Service for CamService {
    fn build(scope: &mut ServiceScope<Self>) {
        scope
            // Inspector camera should be persistent.
            .init_with(
                |mut commands: Commands, query: Query<&InspectorCam>| -> InitResult {
                    if query.is_empty() {
                        let transform =
                            Transform::from_xyz(2., 2., 5.).looking_at(Vec3::ZERO, Vec3::Y);
                        commands.spawn(InspectorCamBundle::new(transform));
                    } else if query.iter().len() > 1 {
                        return Err("Multiple inspector cameras!".into());
                    }
                    Ok(None)
                },
            )
            .on_up(
                |mut cam: Single<&mut Camera, With<InspectorCam>>| -> UpResult {
                    cam.is_active = true;
                    Ok(())
                },
            )
            .on_down(
                |_: In<DownReason>, mut cam: Single<&mut Camera, With<InspectorCam>>| {
                    cam.is_active = false;
                },
            )
            .add_systems(
                Update,
                (Dolly::<InspectorCam>::update_active, update_camera),
            );
    }
}

pub struct InspectorCamPlugin;
impl Plugin for InspectorCamPlugin {
    fn build(&self, app: &mut App) {
        app.register_service::<CamService>();
    }
}
