// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod bundle;
pub use bundle::*;

use bevy::prelude::*;
use q_utils::boolish_states;

use crate::prelude::*;

boolish_states!(PlayerState);

pub struct IntegrationPlugin;
impl IntegrationPlugin {
    // TODO: This position needs to vary depending on the terrain. Probably want
    // to wait until it's loaded. But, game state should wait until terrain
    // is loaded to transition to PlayerState::Enabled
    fn spawn(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let pos = Vec3::ZERO;
        let capsule = meshes.add(Capsule3d::new(0.5, 1.));
        let material = materials.add(StandardMaterial::default());
        commands.spawn(PlayerBundle::new(
            Transform::from_translation(pos),
            capsule,
            material.clone(),
        ));
        commands.spawn(PlayerCamBundle::new());
    }
}
impl Plugin for IntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.setup_boolish_states()
            .add_systems(OnExit(PlayerState::Init), Self::spawn);
    }
}
