// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_dolly::prelude::*;
use q_utils::InspectorIgnore;

#[derive(Component, Debug, Default)]
pub struct InspectorCam;

pub struct InspectorCamBundle;
impl InspectorCamBundle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(transform: Transform) -> impl Bundle {
        (
            Camera3d::default(),
            transform,
            Name::new("Inspector Cam"),
            InspectorCam,
            InspectorIgnore,
            Rig::builder()
                .with(Fpv::from_position_target(transform))
                .build(),
        )
    }
}

// https://github.com/BlackPhlox/bevy_dolly/blob/main/examples/fpv.rs
pub(crate) fn update_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    mut rig: Single<&mut Rig, With<InspectorCam>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    can_scroll: Res<State<InspectorCamScrollStates>>,
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
    if can_scroll.get().is_enabled() && mouse_btn.pressed(MouseButton::Right) {
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
