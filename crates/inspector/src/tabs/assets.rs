//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{asset::ReflectAsset, reflect::TypeRegistry};
use bevy_egui::egui;

use crate::state::InspectorSelection;

use super::TabViewer;

pub fn render_tab(
    tab_viewer: &mut TabViewer,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration.type_info().type_path_table().short_path(),
                registration.type_id(),
                reflect_asset,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, ..), (name_b, ..)| name_a.cmp(name_b));

    for (asset_name, asset_type_id, reflect_asset) in assets {
        let handles: Vec<_> = reflect_asset.ids(tab_viewer.world).collect();

        ui.collapsing(format!("{asset_name} ({})", handles.len()), |ui| {
            for handle in handles {
                let selected = match tab_viewer.selection {
                    InspectorSelection::Asset(_, _, selected_id) => {
                        selected_id == handle
                    }
                    _ => false,
                };

                if ui
                    .selectable_label(selected, format!("{:?}", handle))
                    .clicked()
                {
                    tab_viewer.selection = InspectorSelection::Asset(
                        asset_type_id,
                        asset_name.to_string(),
                        handle,
                    );
                }
            }
        });
    }
}
