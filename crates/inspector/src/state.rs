//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use std::any::TypeId;

use bevy::{
    asset::{LoadedFolder, UntypedAssetId},
    prelude::*,
};
use bevy_egui::egui::{self};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use egui_dock::{DockArea, NodeIndex, Style};

use crate::{
    InspectorEnabled,
    tabs::{Tab, TabViewer},
};

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
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
            DockState::new(vec![Tab::GameView, Tab::NoiseEditor]);
        let tree = dock_state.main_surface_mut();
        let [_game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![Tab::Inspector]);
        let [_game, _heirarchy] =
            tree.split_left(NodeIndex::root(), 0.25, vec![Tab::Hierarchy]);
        let [_game, _bottom] = tree.split_below(
            NodeIndex::root(),
            0.8,
            vec![Tab::Resources, Tab::Assets],
        );
        dock_state
    }
}

#[derive(Resource)]
pub struct UiState {
    // Idea:
    // As long as the inspector is open, the game is in InspectorMode.
    // Game time is paused and main systems are suspended.
    // Once the user hits "play" the ui will either become disabled or
    // the gameview tab will go fullscreen until the simulation is stopped with ESC.
    // This is basically how it is with Unity.
    // InspectorCamera is a flycam.
    // Will need to switch between game states in the main application.
    pub viewport_rect: egui::Rect,
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
    pub assets: Handle<LoadedFolder>,
}
impl UiState {
    pub fn new(assets: Handle<LoadedFolder>) -> Self {
        Self {
            viewport_rect: egui::Rect::NOTHING,
            selection: InspectorSelection::Entities,
            selected_entities: SelectedEntities::default(),
            assets,
        }
    }
    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        world.resource_scope::<DockState, _>(|world, mut dock_state| {
            DockArea::new(&mut dock_state.0)
                .style(Style::from_egui(ctx.style().as_ref()))
                .show(ctx, &mut TabViewer { state: self, world });
        });
    }
    pub fn enabled(world: &mut World) -> bool {
        world.query::<&InspectorEnabled>().single(world).0
    }
}
