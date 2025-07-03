//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---

use crate::{
    tabs::*,
    ui::{file_dialog::UiFileState, layout::Layout},
};
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiPostUpdateSet};
use derivative::Derivative;

#[derive(Resource, Derivative)]
#[derivative(Debug, Default)]
pub struct UiState {
    pub tab_data: TabData,
    #[derivative(Debug = "ignore")]
    pub toasts: egui_notify::Toasts,
    pub file_dialog: egui_file_dialog::FileDialog,
    pub file_dialog_state: UiFileState,
    pub layout: Layout,
}

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UiSystems;

pub struct UiPlugin;
impl UiPlugin {
    pub fn show_ui_system(world: &mut World) {
        let egui_context = world
            .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
            .single(world);
        if egui_context.is_err() {
            warn!("No window.");
            return;
        }
        let mut egui_context = egui_context.unwrap().clone();

        world.resource_scope::<UiState, _>(|world, mut ui_state| {
            Layout::render(&mut ui_state, world, egui_context.get_mut())
        });
    }
}
impl Plugin for UiPlugin {
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
