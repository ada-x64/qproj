//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use crate::tabs::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    EguiContext, EguiPostUpdateSet,
    egui::{self, mutex::Mutex},
};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use egui_dock::{DockArea, NodeIndex, Style};

use super::UiSystems;

// Resources //////////////////////////////////////////////////////////////////

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
    // pub fn enabled(world: &mut World) -> bool {
    //     (*world.resource::<State<InspectorEnabled>>().get()).into()
    // }
}

// Plugin /////////////////////////////////////////////////////////////////////
pub struct UiStatePlugin;
impl UiStatePlugin {
    pub fn show_ui_system(world: &mut World) -> Result<(), BevyError> {
        let egui_context = world
            .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
            .single(world)?;
        let mut egui_context = egui_context.clone();

        world.resource_scope::<UiState, _>(|world, mut ui_state| {
            ui_state.ui(world, egui_context.get_mut())
        });
        Ok(())
    }
}
impl Plugin for UiStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DockState>()
            .init_resource::<UiState>()
            .add_systems(
                PostUpdate,
                (Self::show_ui_system
                    .before(EguiPostUpdateSet::ProcessOutput)
                    .before(bevy_egui::end_pass_system)
                    .before(
                        bevy::transform::TransformSystem::TransformPropagate,
                    ))
                .in_set(UiSystems),
            );
    }
}
