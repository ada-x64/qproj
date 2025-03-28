use bevy::{input::mouse::MouseMotion, prelude::*, window::PrimaryWindow};
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
    windows: Query<&Window, With<PrimaryWindow>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut rig_q: Query<&mut Rig>,
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
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }
    delta.x *= sensitivity.x;
    delta.y *= sensitivity.y;

    let mut rig = rig_q.single_mut();

    if let Ok(window) = windows.get_single() {
        if !window.cursor_options.visible {
            rig.driver_mut::<Fpv>().update_pos_rot(
                move_vec,
                delta,
                false,
                boost,
                time_delta_seconds,
            );
        }
    }
}
