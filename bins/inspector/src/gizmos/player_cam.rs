//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::color::palettes::css::RED;
pub use bevy::prelude::*;
use q_player::prelude::*;

use super::GizmosPlugin;

impl GizmosPlugin {
    pub fn draw_cam_gizmo(
        mut gizmos: Gizmos,
        q: Query<&Transform, With<PlayerCam>>,
    ) -> Result<(), BevyError> {
        let cam = q.single()?;
        let start_point = cam.translation - cam.forward().as_vec3();
        let end_point = cam.translation;
        gizmos.arrow(start_point, end_point, RED);
        gizmos.sphere(Isometry3d::from_translation(start_point), 0.5, RED);
        Ok(())
    }
}
