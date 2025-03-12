pub mod state;
pub mod tabs;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiPostUpdateSet};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use state::UiState;
use tabs::game_view::set_camera_viewport;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultInspectorConfigPlugin, bevy_egui::EguiPlugin))
            .init_resource::<UiState>()
            .add_systems(
                PostUpdate,
                show_ui_system
                    .before(EguiPostUpdateSet::ProcessOutput)
                    .before(bevy_egui::end_pass_system)
                    .before(
                        bevy::transform::TransformSystem::TransformPropagate,
                    ),
            )
            .add_systems(PostUpdate, set_camera_viewport.after(show_ui_system));
    }
}

fn show_ui_system(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    world.resource_scope::<UiState, _>(|world, mut ui_state| {
        ui_state.ui(world, egui_context.get_mut())
    });
}
