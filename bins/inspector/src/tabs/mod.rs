//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{asset::UntypedAssetId, prelude::*};
use bevy_egui::egui::{self, mutex::Mutex};
use game_view::set_camera_viewport;
use std::any::TypeId;

use crate::prelude::*;

pub mod assets;
pub mod game_view;
pub mod hierarchy;
pub mod inspector;
pub mod resources;

#[derive(Debug, Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Debug)]
pub enum Tab {
    GameView,
    Inspector,
    Hierarchy,
    Resources,
    Assets,
    NoiseEditor,
    States,
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
        let type_registry = self
            .world
            .get_resource::<AppTypeRegistry>()
            .expect("Could not get app type registry!")
            .0
            .clone();
        let type_registry = type_registry.read();
        match tab {
            Tab::GameView => game_view::render_tab(self, ui),
            Tab::Inspector => inspector::render_tab(self, ui, &type_registry),
            Tab::Hierarchy => hierarchy::render_tab(self, ui, &type_registry),
            Tab::Resources => resources::render_tab::<ReflectResource>(
                self,
                ui,
                &type_registry,
            ),
            Tab::States => {
                resources::render_tab::<ReflectState>(self, ui, &type_registry)
            }
            Tab::Assets => assets::render_tab(self, ui, &type_registry),
            Tab::NoiseEditor => todo!(),
        }
    }
}

// Plugin //////////////////////////////////////////////////////////////////////

pub struct TabsPlugin;
impl Plugin for TabsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (set_camera_viewport.after(UiStatePlugin::show_ui_system),)
                .in_set(UiSystems),
        );
    }
}
