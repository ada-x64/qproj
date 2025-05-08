//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};
use bevy_egui::{
    EguiContextSettings,
    egui::{self},
};
use q_utils::BoolishStateTrait;

use crate::prelude::*;

use super::TabViewer;

pub fn render_tab(viewer: &mut TabViewer, ui: &mut egui::Ui) {
    let can_scroll = viewer
        .world
        .get_resource::<State<InspectorCamCanScroll>>()
        .unwrap()
        .as_bool();
    let click_and_drag = ui.interact(
        ui.clip_rect(),
        "gameview_interact".into(),
        egui::Sense::click_and_drag(),
    );

    // Move on click and drag
    if click_and_drag.hovered() && !can_scroll {
        viewer
            .world
            .get_resource_mut::<NextState<InspectorCamCanScroll>>()
            .unwrap()
            .set(true.into());
    } else if !click_and_drag.hovered() && can_scroll {
        viewer
            .world
            .get_resource_mut::<NextState<InspectorCamCanScroll>>()
            .unwrap()
            .set(false.into());
    }

    // TODO: Should we do this in Egui by manually calling a system here?
    // if click_and_drag.dragged() {
    //     let delta = click_and_drag.drag_delta();
    // }

    viewer.state.lock().viewport_rect = ui.clip_rect();
    viewer
        .world
        .resource_scope::<State<GameViewState>, _>(|world, physics| {
            let btn_text = if physics.as_bool() {
                "\u{23f9}"
            } else {
                "\u{25B6}"
            };
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new(btn_text)).clicked() {
                    world
                        .get_resource_mut::<NextState<GameViewState>>()
                        .unwrap()
                        .set(physics.toggle());
                }
            });
        });
}

// make camera only render to view not obstructed by UI
pub fn set_camera_viewport(
    ui_state: Res<UiState>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    egui_settings: Query<&EguiContextSettings>,
    mut cameras: Query<&mut Camera, With<InspectorCam>>,
) {
    let Ok(mut cam) = cameras.get_single_mut() else {
        return;
    };

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    let scale_factor =
        window.scale_factor() * egui_settings.single().scale_factor;

    let viewport_pos =
        ui_state.viewport_rect.left_top().to_vec2() * scale_factor;
    let viewport_size = ui_state.viewport_rect.size() * scale_factor;

    let physical_position =
        UVec2::new(viewport_pos.x as u32, viewport_pos.y as u32);
    let physical_size =
        UVec2::new(viewport_size.x as u32, viewport_size.y as u32);

    // The desired viewport rectangle at its offset in "physical pixel space"
    let rect = physical_position + physical_size;

    let window_size = window.physical_size();
    // wgpu will panic if trying to set a viewport rect which has coordinates extending
    // past the size of the render target, i.e. the physical window in our case.
    // Typically this shouldn't happen- but during init and resizing etc. edge cases might occur.
    // Simply do nothing in those cases.
    if rect.x <= window_size.x && rect.y <= window_size.y {
        cam.viewport = Some(Viewport {
            physical_position,
            physical_size,
            depth: 0.0..1.0,
        });
    } else {
        warn!("Attempted to set camera viewport beyond render target size.")
    }
}
