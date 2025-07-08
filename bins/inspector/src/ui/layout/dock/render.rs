// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub use bevy::prelude::*;
use egui::mutex::Mutex;
use egui_dock::NodeIndex;

use crate::ui::{
    UiState,
    layout::dock::{Tab, tabs},
};

pub struct TabViewer<'a> {
    pub world: &'a mut World,
    pub ui_state: Mutex<&'a mut UiState>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{tab:?}").into()
    }

    fn clear_background(&self, tab: &Self::Tab) -> bool {
        !matches!(tab, Tab::SceneEditor)
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .expect("Could not get app type registry!")
            .0
            .clone();
        let type_registry = type_registry.read();
        match tab {
            Tab::SceneEditor => tabs::scene_editor::render_tab(self, ui),
            Tab::Inspector => {
                tabs::inspector::render_tab(self, ui, &type_registry)
            }
            Tab::Hierarchy => {
                tabs::hierarchy::render_tab(self, ui, &type_registry)
            }
            Tab::Resources => tabs::resources::render_tab::<ReflectResource>(
                self,
                ui,
                &type_registry,
            ),
            Tab::States => tabs::resources::render_tab::<ReflectState>(
                self,
                ui,
                &type_registry,
            ),
            Tab::Assets => tabs::assets::render_tab(self, ui, &type_registry),
            Tab::NoiseEditor => todo!(),
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct DockState(egui_dock::DockState<Tab>);
impl DockState {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self(egui_dock::DockState::new(tabs))
    }
}
impl Default for DockState {
    fn default() -> Self {
        // TODO ? Load layout from disk
        // Set up dock tree.
        let mut dock_state =
            DockState::new(vec![Tab::SceneEditor, Tab::NoiseEditor]);
        let tree = dock_state.main_surface_mut();
        let [_game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![Tab::Inspector]);
        let [_game, _heirarchy] =
            tree.split_left(NodeIndex::root(), 0.25, vec![Tab::Hierarchy]);
        let [_game, _bottom] = tree.split_below(
            NodeIndex::root(),
            0.8,
            vec![Tab::Resources, Tab::Assets, Tab::States],
        );
        dock_state
    }
}
