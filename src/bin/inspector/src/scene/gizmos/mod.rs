// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod axes;
mod player_cam;

use crate::{
    scene::gizmos::{
        axes::{RenderToTextureGroup, render_axes},
        player_cam::draw_cam_gizmo,
    },
    ui::layout::dock::tabs::scene_editor::set_camera_viewport,
};
use bevy::{ecs::system::RunSystemOnce, prelude::*, render::view::RenderLayers};
use q_service::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct GizmosService;
impl Service for GizmosService {
    fn build(scope: &mut ServiceScope<Self>) {
        info!("Building gizmos service!");
        scope.add_systems(
            Update,
            (draw_cam_gizmo, render_axes.after(set_camera_viewport)),
        );
    }
}

pub struct GizmosPlugin;
impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        info!("GizmosPlugin::Build");
        app.register_service::<GizmosService>()
            .init_gizmo_group::<RenderToTextureGroup>();
        {
            let world = app.world_mut();
            let mut gcstore = world.get_resource_mut::<GizmoConfigStore>().unwrap();
            gcstore.insert(
                GizmoConfig {
                    render_layers: RenderLayers::layer(1),
                    ..Default::default()
                },
                RenderToTextureGroup,
            );
            world
                .run_system_once(axes::setup_overlay_ui)
                .expect("Axes should set up properly.");
        }
    }
}
