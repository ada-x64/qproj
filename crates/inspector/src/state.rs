use std::any::TypeId;

use bevy::{asset::UntypedAssetId, prelude::*};
use bevy_egui::egui;
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use egui_dock::{DockArea, DockState, NodeIndex, Style};

use crate::tabs::{Tab, TabViewer};

#[derive(Eq, PartialEq)]
pub enum InspectorSelection {
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
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
    pub inspector_enabled: bool,
    pub dock_state: DockState<Tab>,
    pub viewport_rect: egui::Rect,
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
}
impl UiState {
    pub fn new() -> Self {
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

        Self {
            inspector_enabled: true,
            dock_state,
            viewport_rect: egui::Rect::NOTHING,
            selection: InspectorSelection::Entities,
            selected_entities: SelectedEntities::default(),
        }
    }
    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        let mut tab_viewer = TabViewer {
            world,
            viewport_rect: &mut self.viewport_rect,
            selected_entities: &mut self.selected_entities,
            selection: &mut self.selection,
        };
        DockArea::new(&mut self.dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
