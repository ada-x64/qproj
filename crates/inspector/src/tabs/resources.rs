//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::reflect::{TypeData, TypeRegistry};
use bevy_egui::egui;

use super::{InspectorSelection, TabViewer};

pub fn render_tab<T: TypeData>(
    viewer: &mut TabViewer,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<T>().is_some())
        .map(|registration| {
            (
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    let mut state = viewer.state.lock();
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
