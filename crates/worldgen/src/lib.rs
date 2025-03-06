use bevy::{
    asset::{self, LoadState},
    prelude::*,
};
use chunk::{iter_xy, Chunk, ChunkGenerator, Terrain};
use expr::{Expr, ExprLoader};
pub mod chunk;
mod expr;
pub mod mesh;

#[cfg(test)]
mod chunk_test;

pub struct WorldgenPlugin {
    pub spawn_immediately: bool,
}
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
        commands
            .spawn((Terrain, Transform::default(), Visibility::Visible))
            .with_children(|builder| {
                iter_xy(world_size).for_each(|(x, y)| {
                    let chunk = Chunk::new(&generator, &noise, x, y);
                    let mesh = chunk.to_mesh();
                    builder
                        .spawn((
                            generator.get_transform(x, y),
                            chunk,
                            Visibility::Inherited,
                        ))
                        .with_child((
                            Mesh3d(meshes.add(mesh)),
                            MeshMaterial3d(
                                materials.add(StandardMaterial::default()),
                            ),
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
        .init_asset::<Expr>()
        .init_asset_loader::<ExprLoader>()
        .add_systems(Startup, Self::init)
        // probably want to remove this once it's done...
        .add_systems(Update, (Self::spawn_mesh).run_if(Self::rc_spawn_mesh));
    }
}
