//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{asset::LoadState, prelude::*};
use chunk::{iter_xy, Chunk, ChunkGenerator, Terrain};
use expr::{Expr, ExprLoader};
pub mod chunk;
mod expr;
pub mod mesh;

#[cfg(test)]
mod chunk_test;

#[derive(Resource, Default)]
pub struct WorldgenPluginSettings {
    pub spawn_immediately: bool,
    pub use_debug_colors: bool,
}
pub struct WorldgenPlugin;
impl WorldgenPlugin {
    fn init(assets: Res<AssetServer>, mut generator: ResMut<ChunkGenerator>) {
        let handle = assets.load("terrain/complex-planet.terrain.ron");
        generator.expr_handle = Some(handle.clone());
    }

    fn rc_spawn_mesh(
        server: Res<AssetServer>,
        generator: Res<ChunkGenerator>,
        q: Query<&Terrain>,
    ) -> bool {
        q.is_empty()
            && generator
                .expr_handle
                .clone()
                .and_then(|handle| {
                    server
                        .get_load_state(&handle.untyped())
                        .map(|state| matches!(state, LoadState::Loaded))
                })
                .unwrap_or_default()
    }

    fn spawn_mesh(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        plugin_settings: Res<WorldgenPluginSettings>,
        generator: Res<ChunkGenerator>,
        exprs: Res<Assets<Expr>>,
    ) {
        let Some(expr) = generator
            .expr_handle
            .as_ref()
            .and_then(|handle| exprs.get(handle))
        else {
            warn!("Tried to spawn mesh when expr handle unloaded.");
            return;
        };
        let noise = expr.noise();
        let world_size = 4;
        debug!("Generator settings: {generator:?}");
        commands
            .spawn((Terrain, Transform::default(), Visibility::Visible))
            .with_children(|builder| {
                iter_xy(world_size).for_each(|(x, y)| {
                    debug!("Making chunk at pos ({x},{y})");
                    let chunk = Chunk::new(&generator, &noise, x, y);
                    let mesh = chunk.to_mesh();
                    builder
                        .spawn((
                            generator.get_transform(x, y),
                            chunk,
                            Visibility::Visible,
                            Name::new(format!("chunk ({x},{y})")),
                        ))
                        .with_child((
                            Mesh3d(meshes.add(mesh)),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color: if plugin_settings.use_debug_colors
                                {
                                    Color::hsl(
                                        360. * rand::random::<f32>(),
                                        1.,
                                        1.,
                                    )
                                } else {
                                    Color::WHITE
                                },
                                ..Default::default()
                            })),
                        ));
                })
            });
    }
}
impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkGenerator {
            expr_handle: None,
            max_elevation: 100.,
            scaling_factor: 0.001,
            size: 32,
            seed: 0,
        })
        .init_resource::<WorldgenPluginSettings>()
        .init_asset::<Expr>()
        .init_asset_loader::<ExprLoader>()
        .add_systems(Startup, Self::init)
        // probably want to remove this once it's done...
        .add_systems(Update, (Self::spawn_mesh).run_if(Self::rc_spawn_mesh));
    }
}
