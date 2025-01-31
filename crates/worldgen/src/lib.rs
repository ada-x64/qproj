use bevy::prelude::*;
use mesh::gen_mesh;
pub mod mesh;

#[derive(Resource, Copy, Clone, Debug)]
pub struct WorldgenParameters {
    chunk_size: u32,
}
impl Default for WorldgenParameters {
    fn default() -> Self {
        WorldgenParameters { chunk_size: 32 }
    }
}

pub struct WorldgenPlugin {
    pub spawn_immediately: bool,
}
impl WorldgenPlugin {
    fn init(mut commands: Commands) {
        commands.init_resource::<WorldgenParameters>();
    }
    fn spawn_mesh(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        params: Res<WorldgenParameters>,
    ) {
        let mesh = gen_mesh(*params);

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
        ));
    }
}
impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::init);
        if self.spawn_immediately {
            app.add_systems(Startup, Self::spawn_mesh.after(Self::init));
        }
    }
}
