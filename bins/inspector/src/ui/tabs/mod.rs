//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{asset::UntypedAssetId, prelude::*};
use bevy_egui::egui::{self, mutex::Mutex};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use derivative::Derivative;
use egui_dock::NodeIndex;
use game_view::set_camera_viewport;
use std::any::TypeId;

use crate::prelude::*;

pub mod assets;
pub mod game_view;
pub mod hierarchy;
pub mod inspector;
pub mod resources;

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
            DockState::new(vec![Tab::GameView, Tab::NoiseEditor]);
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

#[derive(Default, Debug, Eq, PartialEq)]
pub enum InspectorSelection {
    #[default]
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

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct TabData {
    #[derivative(Default(value = "egui::Rect::NOTHING"))]
    pub viewport_rect: egui::Rect,
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
    pub show_all_entities: bool,
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
        app.init_resource::<DockState>().add_systems(
            PostUpdate,
            (set_camera_viewport.after(UiPlugin::show_ui_system),)
                .in_set(crate::ui::UiSystems),
        );
    }
}
