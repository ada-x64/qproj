//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
pub mod state;
pub mod tabs;

use avian3d::prelude::*;
use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiPostUpdateSet};
use bevy_flycam::FlyCam;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use state::{DockState, UiState};
use tabs::game_view::{InspectorCamera, set_camera_viewport};

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let assets = app.world_mut().load_asset("inspector");
        app.add_plugins((
            DefaultInspectorConfigPlugin,
            bevy_egui::EguiPlugin,
            bevy_flycam::NoCameraPlayerPlugin,
        ))
        .init_resource::<DockState>()
        .insert_resource(UiState::new(assets))
        .add_systems(Startup, (spawn_camera, spawn_ui))
        .add_systems(FixedUpdate, pause_time)
        .add_systems(
            PostUpdate,
            show_ui_system
                .before(EguiPostUpdateSet::ProcessOutput)
                .before(bevy_egui::end_pass_system)
                .before(bevy::transform::TransformSystem::TransformPropagate),
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

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam,
        InspectorCamera,
    ));
}

#[derive(Component, Deref, DerefMut)]
pub struct InspectorEnabled(pub bool);

fn spawn_ui(mut commands: Commands, mut time: ResMut<Time<Physics>>) {
    time.pause();
    commands.spawn(InspectorEnabled(true));
}

fn pause_time(
    q: Query<&InspectorEnabled, Changed<InspectorEnabled>>,
    mut time: ResMut<Time<Physics>>,
) {
    if let Ok(enabled) = q.get_single() {
        if enabled.0 {
            time.pause();
        } else {
            time.unpause();
        }
    }
}
