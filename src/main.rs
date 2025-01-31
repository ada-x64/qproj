use bevy::{
    app::{App, Startup},
    asset::{Assets, RenderAssetUsages},
    ecs::system::{Commands, ResMut},
    pbr::{MeshMaterial3d, StandardMaterial},
    prelude::*,
    render::mesh::{Indices, Mesh, Mesh3d, PrimitiveTopology},
    DefaultPlugins,
};
use itertools::Itertools;

fn gen_mesh(size: u32) -> Mesh {
    let positions = (0..size * size)
        .map(|idx| [(idx % size) as f32, rand::random(), (idx / size) as f32])
        .collect_vec();
    // todo: probably want to adjust normals to tangent of the current vertex
    let normals = (0..size * size).map(|_idx| [0., 1., 0.]).collect_vec();
    let uvs = (0..size * size)
        .map(|idx| [(idx % 64) as f32 / 255., (idx / 64) as f32 / 255.])
        .collect_vec();

    let mut strip = Vec::new();
    for row in 0..size - 1 {
        if row > 0 {
            strip.push(row * size);
        }
        for col in 0..size - 1 {
            strip.push(row * size + col);
            strip.push((row + 1) * size + col);
        }
        if row < size - 2 {
            strip.push((row + 1) * size + size - 1);
        }
    }

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

fn spawn_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = gen_mesh(32); // 32x32 vertices, 10 units across

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial::default())),
    ));
}

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(bevy_flycam::PlayerPlugin)
        .add_systems(Startup, spawn_mesh)
        .run();
}
