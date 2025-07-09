// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod axes;
mod player_cam;

use crate::{
    scene::gizmos::axes::{RenderToTextureGroup, render_axes},
    state::InspectorSettings,
    ui::layout::dock::tabs::scene_editor,
};
use bevy::{prelude::*, render::view::RenderLayers};
use q_utils::service;

#[derive(
    SystemSet, Default, Reflect, Hash, PartialEq, Eq, Debug, Clone, Copy,
)]
pub struct GizmoSystems;

service!(Gizmos, initialize);

pub struct GizmosPlugin;
impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GizmosServicePlugin)
            .init_gizmo_group::<RenderToTextureGroup>()
            .add_observer(initialize)
            .add_systems(
                Update,
                (
                    Self::draw_cam_gizmo,
                    render_axes.after(scene_editor::set_camera_viewport),
                )
                    .in_set(GizmoSystems),
            )
            .configure_sets(
                Update,
                GizmoSystems.run_if(in_state(GizmosStates::Enabled)),
            );
    }
}

fn initialize(
    _trigger: Trigger<InitGizmos>,
    mut commands: Commands,
    mut gcstore: ResMut<GizmoConfigStore>,
    settings: Res<InspectorSettings>,
) {
    gcstore.insert(
        GizmoConfig {
            render_layers: RenderLayers::layer(1),
            ..Default::default()
        },
        RenderToTextureGroup,
    );
    let id = commands.register_system(axes::setup_overlay_ui);
    commands.run_system(id);
    commands.unregister_system(id);
    commands.trigger(GizmosInitialized(Ok(settings.enable_gizmo_overlay)));
}
