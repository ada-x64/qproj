// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_dolly::prelude::*;

mod bundle;
pub use bundle::*;
use q_utils::{BoolishStateTrait, boolish_states};

boolish_states!(InspectorCamState, InspectorCamCanScroll);
pub struct InspectorCamPlugin;
impl InspectorCamPlugin {
    pub fn spawn_camera(mut commands: Commands) {
        let transform =
            Transform::from_xyz(2., 2., 5.).looking_at(Vec3::ZERO, Vec3::Y);
        commands.spawn(InspectorCamBundle::new(transform));
    }

    // https://github.com/BlackPhlox/bevy_dolly/blob/main/examples/fpv.rs
    pub fn update_camera(
        time: Res<Time>,
        keys: Res<ButtonInput<KeyCode>>,
        mouse_btn: Res<ButtonInput<MouseButton>>,
        mut rig: Single<&mut Rig, With<InspectorCam>>,
        mut mouse_motion_events: EventReader<MouseMotion>,
        can_scroll: Res<State<InspectorCamCanScroll>>,
    ) {
        let time_delta_seconds: f32 = time.delta_secs();
        let boost_mult = 5.0f32;
        let sensitivity = Vec2::splat(1.0);

        let mut move_vec = Vec3::ZERO;

        if keys.pressed(KeyCode::KeyW) {
            move_vec.z -= 1.0;
        }
        if keys.pressed(KeyCode::KeyS) {
            move_vec.z += 1.0;
        }
        if keys.pressed(KeyCode::KeyA) {
            move_vec.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) {
            move_vec.x += 1.0;
        }

        if keys.pressed(KeyCode::KeyE) || keys.pressed(KeyCode::Space) {
            move_vec.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyQ) || keys.pressed(KeyCode::ControlLeft) {
            move_vec.y -= 1.0;
        }

        let boost: f32 = if keys.pressed(KeyCode::ShiftLeft) {
            boost_mult
        } else {
            1.
        };

        let mut delta = Vec2::ZERO;
        if can_scroll.get().as_bool() && mouse_btn.pressed(MouseButton::Right) {
            for event in mouse_motion_events.read() {
                delta += event.delta;
            }
            delta.x *= sensitivity.x;
            delta.y *= sensitivity.y;
        }

        rig.driver_mut::<Fpv>().update_pos_rot(
            move_vec,
            delta,
            true,
            boost,
            time_delta_seconds,
        );
    }
    fn set_cam_active<const VAL: bool>(
        mut cam: Single<&mut Camera, With<InspectorCam>>,
    ) {
        cam.is_active = VAL;
    }
    // /// Grabs/ungrabs mouse cursor
    // fn toggle_grab_cursor(window: &mut Window) -> bool {
    //     match window.cursor_options.grab_mode {
    //         CursorGrabMode::None => {
    //             window.cursor_options.grab_mode = CursorGrabMode::Confined;
    //             window.cursor_options.visible = false;
    //             false
    //         }
    //         _ => {
    //             window.cursor_options.grab_mode = CursorGrabMode::None;
    //             window.cursor_options.visible = true;
    //             true
    //         }
    //     }
    // }
}
impl Plugin for InspectorCamPlugin {
    fn build(&self, app: &mut App) {
        app.setup_boolish_states()
            .add_systems(OnExit(InspectorCamState::Init), Self::spawn_camera)
            .add_systems(
                Update,
                (Dolly::<InspectorCam>::update_active, Self::update_camera)
                    .run_if(in_state(InspectorCamState::Enabled)),
            )
            .add_systems(
                OnEnter(InspectorCamState::Disabled),
                Self::set_cam_active::<false>,
            )
            .add_systems(
                OnEnter(InspectorCamState::Enabled),
                Self::set_cam_active::<true>,
            );
    }
}
