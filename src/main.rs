use bevy::{
    app::{App, Startup},
    asset::{Assets, RenderAssetUsages},
    ecs::system::{Commands, ResMut},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::*,
    render::mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology},
    DefaultPlugins,
};
use bevy_flycam::FlyCam;
use itertools::Itertools;

fn gen_strip(size: u32) -> Vec<u32> {
    let mut strip = Vec::new();
    for row in 0..size - 1 {
        if row > 0 {
            strip.push(row * size);
        }
        for col in 0..size {
            strip.push(row * size + col);
            strip.push(((row + 1) * size) + col);
        }
        if row < size - 2 {
            strip.push((row + 1) * size + (size - 1));
        }
    }
    strip
}
#[test]
fn test_stip() {
    let strip = gen_strip(4);
    //  0  1  2  3
    //  4  5  6  7
    //  8  9 10 11
    // 12 13 14 15
    #[rustfmt::skip]
    let correct = vec![
           0,  4, 1,  5,  2,  6,  3,  7,  7,
        4, 4,  8, 5,  9,  6, 10,  7, 11, 11,
        8, 8, 12, 9, 13, 10, 14, 11, 15
        ];
    assert_eq!(strip, correct);
}

fn gen_mesh(size: u32) -> Mesh {
    let positions = (0..size * size)
        .map(|idx| [(idx % size) as f32, rand::random(), (idx / size) as f32])
        .collect_vec();
    // todo: probably want to adjust normals to tangent of the current vertex
    let normals = (0..size * size).map(|_idx| [0., 1., 0.]).collect_vec();
    let uvs = (0..size * size)
        .map(|idx| [(idx % 64) as f32 / 255., (idx / 64) as f32 / 255.])
        .collect_vec();

    let strip = gen_strip(size);

    // should also generate normals and uvs
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(strip));

    mesh
}

#[derive(Component)]
struct CamLight;

fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = gen_mesh(256);

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
            light_transform.translation = cam_transform.translation;
            light_transform.rotation = cam_transform.rotation;
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
