//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{prelude::*, reflect::TypeRegistry};
use bevy_egui::egui;

use bevy_inspector_egui::bevy_inspector::hierarchy::Hierarchy;
use q_utils::InspectorIgnore;

use crate::ui::layout::dock::{InspectorSelection, TabViewer};

pub fn render_tab(
    viewer: &mut TabViewer,
    ui: &mut egui::Ui,
    type_registry: &TypeRegistry,
) {
    let mut state = viewer.ui_state.lock();
    let state = &mut state.tab_data;
    let show_all = &mut state.show_all_entities;
    let text = if *show_all {
        "Show scene only"
    } else {
        "Show all"
    };
    ui.toggle_value(show_all, text).clicked();

    let show_all = *show_all;
    let selected = &mut state.selected_entities;
    let mut hierarchy = Hierarchy {
        world: viewer.world,
        type_registry,
        selected,
        context_menu: None,
        shortcircuit_entity: None,
        extra_state: &mut (),
    };
    let selected = if show_all {
        hierarchy.show_with_default_filter::<(
            Without<InspectorIgnore>,
            Without<ChildOf>,
        )>(ui)
    } else {
        hierarchy.show_with_default_filter::<With<DynamicSceneRoot>>(ui)
    };

    if selected {
        state.selection = InspectorSelection::Entities;
    }
}
