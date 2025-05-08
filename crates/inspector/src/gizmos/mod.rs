mod player_cam;

use bevy::prelude::*;

pub struct GizmosPlugin;
impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::draw_cam_gizmo);
    }
}
