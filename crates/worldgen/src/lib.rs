//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{block_on, futures_lite::future, AsyncComputeTaskPool, Task},
};
use chunk::Chunk;
use expr::{Expr, ExprLoader};
use generator::ChunkGenerator;
use util::{iter_xy, Callback, CallbackTriggered, Terrain};
pub mod chunk;
mod expr;
pub mod mesh;

#[cfg(test)]
mod chunk_test;
pub mod generator;
pub mod util;

#[derive(Resource, Default)]
pub struct WorldgenPluginSettings {
    pub spawn_immediately: bool,
    pub use_debug_colors: bool,
}

#[derive(Event)]
pub struct InitTerrain;

#[derive(Component)]
pub struct ComputeChunk(pub Task<CommandQueue>);

pub struct WorldgenPlugin;
impl WorldgenPlugin {
    fn init(
        assets: Res<AssetServer>,
        mut generator: ResMut<ChunkGenerator>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let expr = assets.load("terrain/complex-planet.terrain.ron");
        generator.expr = Some(expr);
        let material = materials.add(StandardMaterial::default());
        generator.default_material = Some(material);
        debug!("INIT");
    }

    fn init_oneshots(mut commands: Commands) {
        // let id = commands.register_system(Self::init_terrain);
        // commands.spawn((InitTerrain, Callback(id)));
    }

    fn evaluate_triggers(
        mut commands: Commands,
        q: Option<Single<&Terrain>>,
        server: Res<AssetServer>,
        generator: Res<ChunkGenerator>,
    ) {
        let loaded = (|| {
            let state = server.get_load_state(generator.expr.as_ref()?)?;
            Some(state.is_loaded())
        })()
        .unwrap_or_default();

        let run_init_terrain = q.is_none() && loaded;
        if run_init_terrain {
            debug!("TRIGGER INIT_TERRAIN");
            commands.trigger(InitTerrain);
        }
    }

    fn run_cbs(
        mut commands: Commands,
        q: Query<(Entity, &Callback), With<CallbackTriggered>>,
    ) {
        q.iter().for_each(|(entt, cb)| {
            commands.run_system(cb.0);
            commands.entity(entt).remove::<CallbackTriggered>();
        })
    }

    fn init_terrain(
        _trigger: Trigger<InitTerrain>,
        mut commands: Commands,
        generator: Res<ChunkGenerator>,
        exprs: Res<Assets<Expr>>,
    ) {
        let pool = AsyncComputeTaskPool::get();
        let world_size = 4;
        let terrain = commands
            .spawn((
                Terrain,
                Transform::default(),
                Visibility::Visible,
                Name::new("Terrain"),
            ))
            .id();
        iter_xy(world_size).for_each(|(x, y)| {
            debug!("{x},{y}");
            let data = generator
                .get_data(&exprs)
                .expect("Couldn't create generator data");
            let mut task_child = commands.spawn_empty();
            let task_child_id = task_child.id();
            let task = pool.spawn(async move {
                let transform = data.get_transform(x, y);
                let chunk = Chunk::new(data, x, y);
                let mesh = chunk.to_mesh();
                debug!("MADE CHUNK AT ({x},{y})");
                let mut command_queue = CommandQueue::default();
                command_queue.push(move |world: &mut World| {
                    debug!("SPAWNING CHUNK AT ({x},{y})");
                    let default_material = world
                        .resource_mut::<ChunkGenerator>()
                        .default_material
                        .clone()
                        .expect("default_material was none!");

                    let mesh = world.resource_mut::<Assets<Mesh>>().add(mesh);

                    let chunk_entt = world
                        .spawn((
                            transform,
                            chunk,
                            Visibility::Visible,
                            Name::new(format!("chunk ({x},{y})")),
                        ))
                        .with_child((
                            Mesh3d(mesh),
                            MeshMaterial3d(default_material),
                        ))
                        .id();
                    world.entity_mut(terrain).add_child(chunk_entt);
                    world.entity_mut(task_child_id).despawn();
                });
                command_queue
            });
            task_child.insert(ComputeChunk(task));
            commands.entity(terrain).add_child(task_child_id);
        });
    }

    fn handle_tasks(mut commands: Commands, mut q: Query<&mut ComputeChunk>) {
        // https://github.com/bevyengine/bevy/blob/adbb53b87f146b8750cb932ca4deb4f875d3e6b6/examples/async_tasks/async_compute.rs#L111
        // Iter through all ComputeChunk instances, poll their tasks, and if they're complete then run their command queue
        for mut task in &mut q {
            if let Some(mut queue) = block_on(future::poll_once(&mut task.0)) {
                commands.append(&mut queue);
            }
        }
    }
}
impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkGenerator>()
            .init_resource::<WorldgenPluginSettings>()
            .init_asset::<Expr>()
            .init_asset_loader::<ExprLoader>()
            .add_event::<InitTerrain>()
            .add_observer(Self::init_terrain)
            .add_systems(Startup, (Self::init, Self::init_oneshots))
            .add_systems(
                Update,
                (Self::evaluate_triggers, Self::run_cbs, Self::handle_tasks)
                    .chain(),
            );
    }
}
