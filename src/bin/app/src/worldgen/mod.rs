// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

use avian3d::prelude::*;
use bevy::{
    ecs::world::CommandQueue,
    prelude::*,
    tasks::{AsyncComputeTaskPool, block_on, futures_lite::future},
};
#[cfg(feature = "inspector")]
use bevy_inspector_egui::InspectorOptions;
use chunk::Chunk;
use expr::{Expr, ExprLoader};
use generator::{ChunkGenerationData, ChunkGenerator};
use itertools::Itertools;
use util::{
    Callback, CallbackTriggered, ComputeChunk, Initialized, SpawnAround, SpawnAroundTracker,
    Terrain, TerrainIntialized, euclidean_dist, iter_xy,
};
pub mod chunk;
mod expr;
pub mod mesh;

#[cfg(test)]
mod chunk_test;
pub mod generator;
pub mod util;

#[derive(Resource, Reflect, Default)]
#[cfg_attr(feature = "inspector", derive(InspectorOptions))]
pub struct WorldgenPluginSettings {
    pub spawn_immediately: bool,
    pub use_debug_colors: bool,
}

pub struct WorldgenPlugin;
impl WorldgenPlugin {
    fn evaluate_triggers(
        mut commands: Commands,
        q: Option<Single<&Terrain, Without<Initialized>>>,
        server: Res<AssetServer>,
        generator: Res<ChunkGenerator>,
    ) {
        let loaded = (|| {
            let state = server.get_load_state(&generator.expr)?;
            Some(state.is_loaded())
        })()
        .unwrap_or_default();

        if q.is_some() && loaded {
            commands.entity(generator.terrain).insert(Initialized);
            commands.trigger(TerrainIntialized);
            debug!("TERRAIN INITIALIZED");
        }
    }

    fn run_cbs(mut commands: Commands, q: Query<(Entity, &Callback), With<CallbackTriggered>>) {
        q.iter().for_each(|(entt, cb)| {
            commands.run_system(cb.0);
            commands.entity(entt).remove::<CallbackTriggered>();
        })
    }

    async fn spawn_chunk(
        pos: IVec2,
        data: ChunkGenerationData,
        task_child_id: Entity,
    ) -> CommandQueue {
        let transform = data.get_transform(pos);
        let chunk = Chunk::new(data, pos, 1.);
        let mesh = chunk.to_mesh();
        // debug!("MADE CHUNK AT ({pos})");
        let mut command_queue = CommandQueue::default();
        command_queue.push(move |world: &mut World| {
            // debug!("SPAWNING CHUNK AT ({pos})");
            let default_material = world
                .get_resource_mut::<ChunkGenerator>()
                .map(|g| g.material.clone());
            if default_material.is_none() {
                error!("Could not get chunk generator's default material!");
                return;
            }
            let collider =
                Collider::trimesh_from_mesh(&mesh).expect("Could not create chunk collider");
            let default_material = default_material.unwrap();
            let mesh_handle = world
                .get_resource_mut::<Assets<Mesh>>()
                .expect("Could not get mesh assets!")
                .add(mesh);
            let chunk_entt = world.spawn((
                transform,
                chunk,
                Visibility::Visible,
                Name::new(format!("chunk {pos}")),
                MeshMaterial3d(default_material),
                Mesh3d(mesh_handle),
                collider,
                RigidBody::Static,
            ));

            let chunk_id = chunk_entt.id();
            let terrain = world
                .get_resource_mut::<ChunkGenerator>()
                .expect("No chunk generator!")
                .terrain;
            world
                .get_entity_mut(terrain)
                .expect("Could not get terrain entity!")
                .add_child(chunk_id);
            world
                .get_entity_mut(task_child_id)
                .expect("Could not get task child entity!")
                .despawn();
        });
        command_queue
    }

    /// Spawns chunks around a given point.
    /// E.g. given `(2,3,3)`
    /// ```ignore
    ///    (1,2) (2,2) (3,2) -> (2+0-1, 3+0-1) (2+1-1, 3+1-1) etc.
    ///    (1,3) (2,3) (3,3)
    ///    (1,4) (2,4) (3,4)
    /// ```
    fn spawn_around(
        trigger: Trigger<SpawnAround>,
        mut commands: Commands,
        generator: ResMut<ChunkGenerator>,
        exprs: Res<Assets<Expr>>,
        chunks: Query<(Entity, &Chunk)>,
    ) {
        let mut to_populate = iter_xy(generator.opts.active_radius, trigger.pos)
            .sorted_by(|p1, p2| {
                let d1 = euclidean_dist(*p1, trigger.pos);
                let d2 = euclidean_dist(*p2, trigger.pos);
                std::cmp::PartialOrd::partial_cmp(&d1, &d2).unwrap()
            })
            .collect_vec();
        let to_modify = chunks.iter().filter_map(|(entt, chunk)| {
            if let Some((idx, _pos)) = to_populate.iter().find_position(|pos| **pos == chunk.pos) {
                to_populate.swap_remove(idx);
                let dist = euclidean_dist(chunk.pos, trigger.pos);
                if dist > generator.opts.lod_cutoff as f32 {
                    Some((entt, chunk, Some(dist)))
                } else {
                    None
                }
            } else {
                Some((entt, chunk, None))
            }
        });

        let pool = AsyncComputeTaskPool::get();
        to_modify.for_each(|(entt, _chunk, maybe_dist)| {
            if let Some(_dist) = maybe_dist {
                // generator.get_data(&exprs);
                // pool.spawn(|| {
                //     commands.entity(entt).remove::<Mesh3d>().insert();
                // })
                // TODO
            } else {
                commands.entity(entt).despawn();
            }
        });

        to_populate.into_iter().for_each(|pos| {
            let data = generator
                .get_data(&exprs)
                .expect("Couldn't create generator data");
            let mut task_child = commands.spawn_empty();
            let task_child_id = task_child.id();
            let task = pool.spawn(Self::spawn_chunk(pos, data, task_child_id));
            task_child.insert(ComputeChunk(task));
            commands.entity(generator.terrain).add_child(task_child_id);
        });
    }

    fn trigger_spawn_around(
        mut commands: Commands,
        mut generator: ResMut<ChunkGenerator>,
        tf: Query<(&SpawnAroundTracker, &Transform)>,
    ) -> Result<(), BevyError> {
        let tf = tf.single()?;
        let pos = generator.world_pos_to_chunk_pos(tf.1.translation.xz());
        if pos != generator.current_chunk {
            generator.current_chunk = pos;
            commands.trigger(SpawnAround { pos })
        }
        Ok(())
    }

    fn handle_tasks(mut commands: Commands, mut q: Query<&mut ComputeChunk>) {
        // https://github.com/bevyengine/bevy/blob/adbb53b87f146b8750cb932ca4deb4f875d3e6b6/examples/async_tasks/async_compute.rs#L111
        // Iter through all ComputeChunk instances, poll their tasks, and if
        // they're complete then run their command queue
        for mut task in &mut q {
            if let Some(mut queue) = block_on(future::poll_once(&mut task.0)) {
                commands.append(&mut queue);
            }
        }
    }

    /// TODO: Make this an event.
    fn spawn_around_run_condition(
        terrain: Query<&Terrain, With<Initialized>>,
        tracker: Query<&SpawnAroundTracker, With<Transform>>,
    ) -> bool {
        terrain.single().is_ok() && tracker.single().is_ok()
    }
}

impl Plugin for WorldgenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldgenPluginSettings>()
            .register_type::<WorldgenPluginSettings>()
            .init_asset::<Expr>()
            .init_asset_loader::<ExprLoader>()
            // NOTE: the FromWorld impl requires ExprLoader to be initialized
            .init_resource::<ChunkGenerator>()
            .register_type::<ChunkGenerator>()
            .add_event::<SpawnAround>()
            .add_observer(Self::spawn_around)
            .add_systems(
                Update,
                (
                    (Self::evaluate_triggers, Self::run_cbs, Self::handle_tasks).chain(),
                    (Self::trigger_spawn_around).run_if(Self::spawn_around_run_condition),
                ),
            );
    }
}
