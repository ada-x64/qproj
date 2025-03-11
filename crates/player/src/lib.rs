use bevy::prelude::*;
use bevy_flycam::FlyCam;
#[cfg(feature = "debug")]
use debug_gizmos::{DebugBundle, DebugLevel, ShowAxes};

#[derive(Component, Default, Debug)]
pub struct Player;

pub struct PlayerPlugin {
    pub enable_flycam: bool,
}

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
            light_transform.rotation = cam_transform.rotation;
            light_transform.translation = cam_transform.translation;
        }
    }
}

pub fn startup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(Capsule3d::new(0.5, 1.));
    commands.spawn((Player, Mesh3d(mesh)));
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, startup);
        if self.enable_flycam {
            app.add_plugins(bevy_flycam::PlayerPlugin)
                .add_systems(Startup, spawn_light)
                .add_systems(Update, light_follows_camera);
        }
    }
}
