//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
pub mod cam;
pub mod state;
pub mod tabs;

use avian3d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_dolly::prelude::*;
use bevy_egui::{EguiContext, EguiPostUpdateSet};
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use cam::{InspectorCam, spawn_camera, update_camera};
use state::{DockState, InspectorState, UiState};
use tabs::game_view::set_camera_viewport;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UISet;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let assets = app.world_mut().load_asset("inspector");
        app.add_plugins((DefaultInspectorConfigPlugin, bevy_egui::EguiPlugin))
            .init_resource::<DockState>()
            .insert_resource(UiState::new(assets))
            .insert_state(InspectorState::Init)
            .add_systems(
                OnTransition {
                    exited: InspectorState::Init,
                    entered: InspectorState::Enabled,
                },
                (|| debug!("ENABLING INSPECTOR UI!"), spawn_camera),
            )
            .add_systems(
                Update,
                (Dolly::<InspectorCam>::update_active, update_camera)
                    .run_if(in_state(InspectorState::Enabled)),
            )
            .add_systems(OnEnter(InspectorState::Disabled), unpause_time)
            .add_systems(OnEnter(InspectorState::Enabled), pause_time);

        app.add_systems(
            PostUpdate,
            (show_ui_system
                .before(EguiPostUpdateSet::ProcessOutput)
                .before(bevy_egui::end_pass_system)
                .before(bevy::transform::TransformSystem::TransformPropagate))
            .in_set(UISet),
        )
        .add_systems(
            PostUpdate,
            (set_camera_viewport.after(show_ui_system),).in_set(UISet),
        )
        .configure_sets(
            PostUpdate,
            UISet.run_if(
                in_state(InspectorState::Enabled)
                    .or(in_state(InspectorState::Disabled)),
            ),
        );
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

fn pause_time(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

fn unpause_time(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}
