//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{ecs::reflect::ReflectResource, reflect::TypeRegistry};
use bevy_egui::egui;

use crate::state::InspectorSelection;

use super::TabViewer;

pub fn render_tab(
    tab_viewer: &mut TabViewer,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    let mut state = tab_viewer.state.lock();
    for (resource_name, type_id) in resources {
        let selected = match state.selection {
            InspectorSelection::Resource(selected, _) => selected == type_id,
            _ => false,
        };

        if ui.selectable_label(selected, resource_name).clicked() {
            state.selection = InspectorSelection::Resource(
                type_id,
                resource_name.to_string(),
            );
        }
    }
}
