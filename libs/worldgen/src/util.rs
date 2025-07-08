// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{
    ecs::{system::SystemId, world::CommandQueue},
    prelude::*,
    tasks::Task,
};

/// Takes a size, squares it, and returns a map with (x,y) coordinates centered
/// around the given point.
pub fn iter_xy(radius: i32, center: IVec2) -> impl Iterator<Item = IVec2> {
    let r = radius;
    let d = 2 * r;
    let (cx, cy) = (center.x, center.y);
    (0..d * d).map(move |i| IVec2::new(cx + i % d - r, cy + i / d - r))
}

/// Returns an iterator of (x,y) coordinates that fit within a circle of the
/// given radius centered around the passed position.
pub fn iter_radius_xy(
    radius: i32,
    center: IVec2,
) -> impl Iterator<Item = IVec2> {
    let r = radius;
    let d = 2 * r;
    let (cx, cy) = (center.x, center.y);
    (0..d * d).filter_map(move |i| {
        let x = i % d - r;
        let y = i / d - r;
        let xx = x - cx;
        let yy = y - cy;
        (xx * xx + yy * yy < r * r).then_some(IVec2::new(cx + x, cy + y))
    })
}

#[derive(Default, Component)]
pub struct Terrain;

#[derive(Component)]
pub struct Callback(pub SystemId);

#[derive(Component)]
pub struct CallbackTriggered;

#[derive(Component, Copy, Clone)]
pub struct SpawnAroundTracker;

#[derive(Event)]
pub struct SpawnAround {
    pub pos: IVec2,
}

#[derive(Event)]
pub struct InitTerrain;

#[derive(Component)]
pub struct Initialized;

#[derive(Component)]
pub struct ComputeChunk(pub Task<CommandQueue>);

pub fn euclidean_dist(p1: IVec2, p2: IVec2) -> f32 {
    let dx = p1.x - p2.x;
    let dy = p1.y - p2.y;
    let sum = (dx * dx) + (dy * dy);
    f32::sqrt(sum as f32)
}

/// Triggers when the terrain struct has been initialized with its assets
#[derive(Event)]
pub struct TerrainIntialized;

/// Triggers when all chunks to be updated or spawned in `spawnAround` are
/// complete
#[derive(Event)]
pub struct ChunksLoaded;
