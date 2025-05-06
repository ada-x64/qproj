//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};
use bevy_dolly::prelude::*;

#[derive(Component, Debug, Default)]
pub struct InspectorCam;

pub fn spawn_camera(mut commands: Commands) {
    let translation = [2.0f32, 2.0f32, 5.0f32];
    let transform = Transform::from_translation(Vec3::from_slice(&translation))
        .looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3d::default(),
        transform,
        InspectorCam,
        Rig::builder()
            .with(Fpv::from_position_target(transform))
            .build(),
    ));
}

// https://github.com/BlackPhlox/bevy_dolly/blob/main/examples/fpv.rs
pub fn update_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    mut cam_rig: Single<(&Camera, &mut Rig), With<InspectorCam>>,
    window: Query<&Window>,
    mut mouse_motion_events: EventReader<MouseMotion>,
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
    // should only activate if the cursor is within the gameview window
    let cursor_pos = window.get_single().ok().and_then(|w| w.cursor_position());
    let (cam, rig) = (cam_rig.0, &mut cam_rig.1);
    let can_scroll = cam
        .viewport
        .as_ref()
        .zip(cursor_pos)
        .map(|(vp, cpos)| {
            let topleft = vp.physical_position.as_vec2();
            let bottomright =
                vp.physical_position.as_vec2() + vp.physical_size.as_vec2();
            topleft.x < cpos.x
                && topleft.y < cpos.y
                && bottomright.x > cpos.x
                && bottomright.y > cpos.y
        })
        .unwrap_or_default();
    if can_scroll && mouse_btn.pressed(MouseButton::Right) {
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
/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) -> bool {
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = false;
            false
        }
        _ => {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            true
        }
    }
}
