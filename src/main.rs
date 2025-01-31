use bevy::{
    app::{App, Startup},
    asset::Assets,
    ecs::system::{Commands, ResMut},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::*,
    render::mesh::{Mesh, Mesh3d},
    DefaultPlugins,
};
use bevy_flycam::FlyCam;
use worldgen::gen_mesh;

#[derive(Component)]
struct CamLight;

fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = gen_mesh(64);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
    ));
    commands.spawn((
        SpotLight {
            range: 1000.,
            shadows_enabled: true,
            ..Default::default()
        },
        CamLight,
        Transform::default(),
    ));
}

fn light_follows_camera(
    cams: Query<&Transform, With<FlyCam>>,
    mut lights: Query<&mut Transform, (With<SpotLight>, Without<FlyCam>)>,
) {
    for cam_transform in &cams {
        for mut light_transform in &mut lights {
            light_transform.set_if_neq(*cam_transform);
        }
    }
}

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_flycam::PlayerPlugin)
        .add_systems(Startup, spawn_mesh)
        .add_systems(Update, light_follows_camera)
        .run();
}
