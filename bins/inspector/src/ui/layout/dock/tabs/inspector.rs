// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::reflect::TypeRegistry;
use bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector::{
    self, ui_for_entities_shared_components, ui_for_entity_with_children,
};

use crate::ui::layout::dock::{InspectorSelection, TabViewer};

pub fn render_tab(
    viewer: &mut TabViewer,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let state = viewer.ui_state.lock();
    let state = &state.tab_data;
    match state.selection {
        InspectorSelection::Entities => {
            match state.selected_entities.as_slice() {
                &[entity] => {
                    ui_for_entity_with_children(viewer.world, entity, ui)
                }
                entities => ui_for_entities_shared_components(
                    viewer.world,
                    entities,
                    ui,
                ),
            }
        }
        InspectorSelection::Resource(type_id, ref name) => {
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_resource(
                viewer.world,
                type_id,
                ui,
                name,
                type_registry,
            )
        }
        InspectorSelection::Asset(type_id, ref name, handle) => {
            ui.label(name);
            bevy_inspector::by_type_id::ui_for_asset(
                viewer.world,
                type_id,
                handle,
                ui,
                type_registry,
            );
        }
    }
}
