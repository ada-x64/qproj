// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod bundle;
pub use bundle::*;

use bevy::prelude::*;
use q_service::prelude::*;

use crate::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct PlayerService;

impl Service for PlayerService {
    fn build(scope: &mut ServiceScope<Self>) {
        scope.init_with(spawn).is_startup(true);
    }
}
pub struct IntegrationPlugin;

impl IntegrationPlugin {}
impl Plugin for IntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.register_service::<PlayerService>();
    }
}

// TODO: This position needs to vary depending on the terrain. Probably want
// to wait until it's loaded. But, game state should wait until terrain
// is loaded to transition to PlayerState::Enabled
fn spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) -> InitResult {
    let pos = Vec3::ZERO;
    let capsule = meshes.add(Capsule3d::new(0.5, 1.));
    let material = materials.add(StandardMaterial::default());
    commands.spawn(PlayerBundle::new(
        Transform::from_translation(pos),
        capsule,
        material.clone(),
    ));
    commands.spawn(PlayerCamBundle::new());
    Ok(None)
}
