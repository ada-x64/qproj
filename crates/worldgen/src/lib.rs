use bevy::{prelude::*, render::primitives::Aabb};
use chunk::ChunkGenerator;
pub mod chunk;
pub mod mesh;

#[derive(Resource, Copy, Clone, Debug, Default)]
pub struct WorldgenParameters {
    chunk_generator: ChunkGenerator,
}
#[derive(Component)]
struct ShowAxes;
fn draw_axes(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &Aabb), With<ShowAxes>>,
) {
    for (&transform, &aabb) in &query {
        let length = aabb.half_extents.length();
        gizmos.axes(transform, length);
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
        let world_size = 3;
        (0..world_size * world_size).for_each(|idx| {
            let x = (idx / world_size) - world_size / 2;
            let y = (idx % world_size) - world_size / 2;
            let transform = params.chunk_generator.get_transform(x, y);
            let chunk = params.chunk_generator.generate(x, y);
            let mesh = chunk.to_mesh();
            commands.spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(materials.add(StandardMaterial::default())),
                transform,
            ));
        })
        // let mesh = gen_mesh(gen_positions(16));
        // commands.spawn((
        //     Mesh3d(meshes.add(mesh)),
        //     MeshMaterial3d(materials.add(StandardMaterial::default())),
        // ));
    }
}
impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::init)
            .add_systems(Update, draw_axes);
        if self.spawn_immediately {
            app.add_systems(Startup, Self::spawn_mesh.after(Self::init));
        }
    }
}
