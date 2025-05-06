//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use bevy_egui::egui::{self, mutex::Mutex};
use bevy_inspector_egui::bevy_inspector::hierarchy::Hierarchy;

use crate::state::{InspectorSelection, UiState};

pub mod assets;
pub mod game_view;
pub mod inspector;
pub mod resources;

#[derive(Debug)]
pub enum Tab {
    GameView,
    Inspector,
    Hierarchy,
    Resources,
    Assets,
    NoiseEditor,
}

pub struct TabViewer<'a> {
    pub world: &'a mut World,
    pub state: Mutex<&'a mut UiState>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Tab::GameView)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let type_registry = self.world.resource::<AppTypeRegistry>().0.clone();
        let type_registry = type_registry.read();
        match tab {
            Tab::GameView => game_view::render_tab(self, ui),
            Tab::Inspector => inspector::render_tab(self, ui, &type_registry),
            Tab::Hierarchy => {
                let mut state = self.state.lock();
                let selected = &mut state.selected_entities;
                let selected = Hierarchy {
                    world: self.world,
                    type_registry: &type_registry,
                    selected,
                    context_menu: None,
                    shortcircuit_entity: None,
                    extra_state: &mut (),
                }
                .show_with_default_filter::<()>(ui);
                // let filter =
                //     Filter::<(Without<Parent>, Without<Observer>)>::from_ui_fuzzy(
                //         ui,
                //         Id::new("fuzzy-filter"),
                //     );
                // ui_for_entities_filtered(self.world, ui, true, &filter);
                if selected {
                    state.selection = InspectorSelection::Entities;
                }
            }
            Tab::Resources => resources::render_tab(self, ui, &type_registry),
            Tab::Assets => assets::render_tab(self, ui, &type_registry),
            Tab::NoiseEditor => todo!(),
        }
    }
}
