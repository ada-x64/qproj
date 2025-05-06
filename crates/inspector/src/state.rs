//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use std::any::TypeId;

use bevy::{asset::UntypedAssetId, prelude::*};
use bevy_egui::egui::{self, mutex::Mutex};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use egui_dock::{DockArea, NodeIndex, Style};
use q_utils::boolish_states;

use crate::tabs::{Tab, TabViewer};

// a bunch of state enums and such ////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Hash, Debug, Resource, Reflect)]
#[reflect(Resource)]
pub struct InspectorSettings {
    pub switch_cams: bool,
}
impl Default for InspectorSettings {
    fn default() -> Self {
        Self { switch_cams: true }
    }
}

boolish_states!(InspectorEnabled, GameViewActive, CamEnabled);

#[derive(Debug, Eq, PartialEq)]
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
            vec![Tab::Resources, Tab::Assets, Tab::States],
        );
        dock_state
    }
}

#[derive(Resource, Debug)]
pub struct UiState {
    pub viewport_rect: egui::Rect,
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
}
impl Default for UiState {
    fn default() -> Self {
        Self {
            viewport_rect: egui::Rect::NOTHING,
            selection: InspectorSelection::Entities,
            selected_entities: SelectedEntities::default(),
        }
    }
}
impl UiState {
    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        world.resource_scope::<DockState, _>(|world, mut dock_state| {
            DockArea::new(&mut dock_state.0)
                .style(Style::from_egui(ctx.style().as_ref()))
                .show(
                    ctx,
                    &mut TabViewer {
                        world,
                        state: Mutex::new(self),
                    },
                );
        });
    }
    pub fn enabled(world: &mut World) -> bool {
        (*world.resource::<State<InspectorEnabled>>().get()).into()
    }
}

// set up the app /////////////////////////////////////////////////////////////

pub trait SetupStates {
    fn setup_states(&mut self) -> &mut Self;
}
impl SetupStates for App {
    fn setup_states(&mut self) -> &mut Self {
        self.setup_boolish_states()
            .init_resource::<DockState>()
            .init_resource::<UiState>()
            .init_resource::<InspectorSettings>()
            .register_type::<InspectorSettings>()
    }
}
