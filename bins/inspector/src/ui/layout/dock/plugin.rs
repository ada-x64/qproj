// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiContextPass};
use egui::mutex::Mutex;
use tiny_bail::prelude::*;

use crate::ui::{
    UiState, UiSystems,
    layout::dock::{
        DockState, TabViewer, tabs::scene_editor::set_camera_viewport,
    },
};

pub struct DockPlugin;
impl Plugin for DockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DockState>()
            .add_systems(PostUpdate, set_camera_viewport)
            .add_systems(EguiContextPass, (render).in_set(UiSystems));
    }
}

/// Renders the DockArea, which renders each Tab
/// through the TabViewer.
fn render(world: &mut World) {
    world.resource_scope(|world, mut ui_state: Mut<UiState>| {
        world.resource_scope(|world, mut dock_state: Mut<DockState>| {
            let mut ctx =
                world.query_filtered::<&mut EguiContext, With<PrimaryWindow>>();
            let ctx = r!(ctx.single_mut(world));
            let mut ctx = ctx.clone();
            let ctx = ctx.get_mut();

            egui_dock::DockArea::new(dock_state.as_mut())
                .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
                .show(
                    ctx,
                    &mut TabViewer {
                        world,
                        ui_state: Mutex::new(ui_state.as_mut()),
                    },
                );
        })
    });
}
