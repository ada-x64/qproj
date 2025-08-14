// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

pub struct ControlsPlugin;
impl ControlsPlugin {
    pub fn apply_controls(
        keyboard: Res<ButtonInput<KeyCode>>,
        mut query: Query<&mut TnuaController>,
    ) -> Result<(), BevyError> {
        let mut controller = query.single_mut()?;

        let mut direction = Vec3::ZERO;

        if keyboard.pressed(KeyCode::ArrowUp) {
            direction -= Vec3::Z;
        }
        if keyboard.pressed(KeyCode::ArrowDown) {
            direction += Vec3::Z;
        }
        if keyboard.pressed(KeyCode::ArrowLeft) {
            direction -= Vec3::X;
        }
        if keyboard.pressed(KeyCode::ArrowRight) {
            direction += Vec3::X;
        }

        // Feed the basis every frame. Even if the player doesn't move - just
        // use `desired_velocity: Vec3::ZERO`. `TnuaController` starts
        // without a basis, which will make the character collider
        // just fall.
        controller.basis(TnuaBuiltinWalk {
            // The `desired_velocity` determines how the character will move.
            desired_velocity: direction.normalize_or_zero() * 10.0,
            desired_forward: Dir3::new(direction.normalize_or_zero()).ok(),
            // The `float_height` must be greater (even if by little) from the
            // distance between the character's center and the
            // lowest point of its collider.
            float_height: 1.5,
            // `TnuaBuiltinWalk` has many other fields for customizing the
            // movement - but they have sensible defaults. Refer to
            // the `TnuaBuiltinWalk`'s documentation to learn what they do.
            ..Default::default()
        });

        // Feed the jump action every frame as long as the player holds the jump
        // button. If the player stops holding the jump button, simply
        // stop feeding the action.
        if keyboard.pressed(KeyCode::Space) {
            controller.action(TnuaBuiltinJump {
                // The height is the only mandatory field of the jump button.
                height: 4.0,
                // `TnuaBuiltinJump` also has customization fields with sensible
                // defaults.
                ..Default::default()
            });
        }

        Ok(())
    }
}
impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_systems(
            FixedUpdate,
            Self::apply_controls.in_set(TnuaUserControlsSystemSet),
        );
    }
}
