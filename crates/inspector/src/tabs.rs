//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use bevy_egui::egui::{self, Id};
use bevy_inspector_egui::bevy_inspector::{Filter, ui_for_entities_filtered};

use crate::state::{InspectorState, UiState};

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

#[derive(Deref, DerefMut)]
pub struct TabViewer<'a> {
    #[deref]
    pub state: &'a mut UiState,
    pub world: &'a mut World,
}
impl TabViewer<'_> {
    pub fn enabled(&mut self) -> bool {
        UiState::enabled(self.world)
    }
    pub fn set_enabled(&mut self, val: bool) {
        self.world
            .resource_mut::<NextState<InspectorState>>()
            .set(val.into());
    }
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
                // let selected = hierarchy_ui(
                //     self.world,
                //     ui,
                //     &mut self.state.selected_entities,
                // );
                // if selected {
                //     self.selection = InspectorSelection::Entities;
                // }
                let filter = Filter::<With<Transform>>::from_ui_fuzzy(
                    ui,
                    Id::new("fuzzy-filter"),
                );
                ui_for_entities_filtered(self.world, ui, true, &filter);
                // if selected {
                //     self.selection = InspectorSelection::Entities;
                // }
            }
            Tab::Resources => resources::render_tab(self, ui, &type_registry),
            Tab::Assets => assets::render_tab(self, ui, &type_registry),
            Tab::NoiseEditor => todo!(),
        }
    }
}
