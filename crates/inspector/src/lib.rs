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
use state::{
    CamEnabled, InspectorEnabled, InspectorSettings, PhysicsEnabled,
    SetupStates, UiState,
};
use tabs::game_view::set_camera_viewport;

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UISet;

pub struct InspectorPlugin;
impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        let _ = {
            app.add_plugins((
                DefaultInspectorConfigPlugin,
                bevy_egui::EguiPlugin,
            ))
            .setup_states()
            .add_systems(OnExit(InspectorEnabled::Init), (init, spawn_camera))
            .add_systems(
                Update,
                (Dolly::<InspectorCam>::update_active, update_camera)
                    .run_if(in_state(CamEnabled::Enabled)),
            )
            .add_systems(OnEnter(CamEnabled::Disabled), set_cam_active::<false>)
            .add_systems(OnEnter(CamEnabled::Enabled), set_cam_active::<true>)
            .add_systems(OnEnter(PhysicsEnabled::Disabled), pause_time)
            .add_systems(OnEnter(PhysicsEnabled::Enabled), unpause_time)
            .add_systems(
                PostUpdate,
                (show_ui_system
                    .before(EguiPostUpdateSet::ProcessOutput)
                    .before(bevy_egui::end_pass_system)
                    .before(
                        bevy::transform::TransformSystem::TransformPropagate,
                    ))
                .in_set(UISet),
            )
            .add_systems(
                PostUpdate,
                (set_camera_viewport.after(show_ui_system),).in_set(UISet),
            )
            .configure_sets(
                PostUpdate,
                UISet.run_if(
                    in_state(InspectorEnabled::Enabled)
                        .or(in_state(InspectorEnabled::Disabled)),
                ),
            )
        };
    }
}

fn init(
    mut physics: ResMut<NextState<PhysicsEnabled>>,
    mut cam: ResMut<NextState<CamEnabled>>,
) {
    physics.set(PhysicsEnabled::Disabled);
    cam.set(CamEnabled::Enabled);
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

fn pause_time(
    mut time: ResMut<Time<Physics>>,
    mut cam: ResMut<NextState<CamEnabled>>,
    settings: Res<InspectorSettings>,
) {
    time.pause();
    if settings.switch_cams {
        cam.set(false.into())
    }
}

fn unpause_time(
    mut time: ResMut<Time<Physics>>,
    mut cam: ResMut<NextState<CamEnabled>>,
) {
    time.unpause();
    cam.set(true.into())
}

fn set_cam_active<const VAL: bool>(
    mut cam: Single<&mut Camera, With<InspectorCam>>,
) {
    cam.is_active = VAL;
}
