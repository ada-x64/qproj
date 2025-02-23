use std::env;

use bevy::{
    app::{App, Startup},
    ecs::system::Commands,
    log::{tracing_subscriber::EnvFilter, LogPlugin},
    prelude::*,
    DefaultPlugins,
};
use bevy_flycam::FlyCam;
#[cfg(feature = "debug")]
use debug_gizmos::{DebugBundle, DebugLevel, DebugPlugin, ShowAxes};

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
        #[cfg(feature = "debug")]
        DebugBundle {
            show_axes: ShowAxes(Some((DebugLevel(0), 3.))),
            ..Default::default()
        },
    ));
}

fn light_follows_camera(
    cams: Query<&Transform, With<FlyCam>>,
    mut lights: Query<&mut Transform, (With<SpotLight>, Without<FlyCam>)>,
) {
    for cam_transform in &cams {
        for mut light_transform in &mut lights {
            // light_transform.set_if_neq(*cam_transform);
            light_transform.rotation = cam_transform.rotation;
        }
    }
}

#[bevy_main]
fn main() {
    let mut app = App::new();

    // use RUST_LOG
    app.add_plugins(DefaultPlugins.set(LogPlugin {
        filter: EnvFilter::from_default_env().to_string(),
        ..Default::default()
    }))
    .add_plugins((
        bevy_flycam::PlayerPlugin,
        worldgen::WorldgenPlugin {
            spawn_immediately: true,
        },
    ))
    .add_systems(Startup, spawn_light)
    .add_systems(Update, light_follows_camera);

    #[cfg(feature = "debug")]
    {
        let level = env::var("DEBUG_LEVEL").unwrap_or_default();
        let debug_level = DebugLevel(level.parse().unwrap_or_default());
        debug!("DEBUG_LEVEL = {debug_level:?}");
        app.add_plugins(DebugPlugin { debug_level });
    }

    app.run();
}
