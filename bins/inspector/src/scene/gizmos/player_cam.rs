// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use q_player::prelude::*;
use tiny_bail::prelude::*;

use super::GizmosPlugin;

impl GizmosPlugin {
    pub fn draw_cam_gizmo(
        mut gizmos: Gizmos,
        q: Query<&Transform, With<PlayerCam>>,
    ) {
        let cam = rq!(q.single());
        let start_point = cam.translation - cam.forward().as_vec3();
        let end_point = cam.translation;
        gizmos.arrow(start_point, end_point, RED);
        gizmos.sphere(Isometry3d::from_translation(start_point), 0.5, RED);
    }
}
