use bevy::{
    app::{App, Startup},
    ecs::system::Commands,
    prelude::*,
    DefaultPlugins,
};
use bevy_flycam::FlyCam;

#[derive(Component)]
struct CamLight;

pub fn spawn_light(mut commands: Commands) {
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
        .add_plugins(worldgen::WorldgenPlugin {
            spawn_immediately: true,
        })
        .add_systems(Startup, spawn_light)
        .add_systems(Update, light_follows_camera)
        .run();
}
