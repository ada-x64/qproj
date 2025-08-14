// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{expr::Expr, util::Terrain};

#[derive(Clone, Debug, Reflect, Asset, Serialize, Deserialize)]
#[cfg_attr(feature = "inspector", derive(InspectorOptions))]
#[cfg_attr(feature = "inspector", reflect(InspectorOptions))]
pub struct ChunkGenOpts {
    /// The length and width of the chunk
    #[cfg_attr(feature = "inspector", inspector(min = 1, max = 128))]
    pub size: usize,
    /// This number is squared to determine the number of chunks spawned around
    /// the player.
    #[cfg_attr(feature = "inspector", inspector(min = 1, max = 16))]
    pub active_radius: i32,
    /// Maximum elevation
    #[cfg_attr(feature = "inspector", inspector(min = -100., max = 100.))]
    pub max_elevation: f64,
    /// The seed for the perlin noise generator
    pub seed: u32,
    /// Perlin noise scaling factor must be a float other than 1.
    /// The larger the number, the smoother the terrain.
    #[cfg_attr(feature = "inspector", inspector(min = 0.0001, max = 0.9999))]
    pub scaling_factor: f64,

    /// How quickly the LOD decreases.
    /// Formula: `size / lod_scale^(dist - lod_cutoff)`
    pub lod_scale: f32,
    /// How many chunks from the player character the LOD begins to decay.
    /// Formula: `size / lod_scale^(dist - lod_cutoff)`
    pub lod_cutoff: i32,
}
impl Default for ChunkGenOpts {
    fn default() -> Self {
        Self {
            max_elevation: 100.,
            active_radius: 4,
            scaling_factor: 0.001,
            size: 64,
            seed: 0,
            lod_scale: 1.5,
            lod_cutoff: 2,
        }
    }
}

#[derive(Debug, Reflect, Resource, Clone)]
#[reflect(Resource)]
pub struct ChunkGenerator {
    pub opts: ChunkGenOpts,
    pub expr: Handle<Expr>,
    pub material: Handle<StandardMaterial>,
    pub terrain: Entity,
    pub current_chunk: IVec2,
}
impl FromWorld for ChunkGenerator {
    fn from_world(world: &mut World) -> Self {
        let expr = world
            .get_resource::<AssetServer>()
            .expect("AssetServer")
            .load("terrain/test.ron");
        let material = world
            .get_resource_mut::<Assets<StandardMaterial>>()
            .expect("AssetServer<StandardMaterial>")
            .add(StandardMaterial::default());
        let terrain = world
            .spawn((
                Terrain,
                Transform::default(),
                Visibility::Visible,
                Name::new("Terrain"),
            ))
            .id();
        Self {
            expr,
            material,
            terrain,
            current_chunk: IVec2::default(),
            opts: ChunkGenOpts::default(),
        }
    }
}

impl ChunkGenerator {
    pub fn get_data(&self, exprs: &Res<Assets<Expr>>) -> Result<ChunkGenerationData, &'static str> {
        let expr = exprs.get(&self.expr).ok_or("Noise expr not initiated")?;
        Ok(ChunkGenerationData {
            max_elevation: self.opts.max_elevation,
            scale: self.opts.scaling_factor,
            size: self.opts.size,
            expr: expr.clone(),
        })
    }
    pub fn world_pos_to_chunk_pos(&self, pos: Vec2) -> IVec2 {
        let x = (pos.x.round() / self.opts.size as f32) as i32;
        let y = (pos.y.round() / self.opts.size as f32) as i32;
        IVec2::new(x, y)
    }
    /// Calculates the number of vertices given the distance from the player.
    pub fn get_num_verts(&self, dist: i32) -> usize {
        if dist > self.opts.lod_cutoff {
            self.opts.size / (2 * (dist - self.opts.lod_cutoff)) as usize
        } else {
            1
        }
    }
}

#[derive(Clone)]
pub struct ChunkGenerationData {
    pub size: usize,
    pub scale: f64,
    pub max_elevation: f64,
    pub expr: Expr,
}
impl ChunkGenerationData {
    pub fn get_transform(&self, pos: IVec2) -> Transform {
        Transform {
            translation: Vec3::new(
                (pos.x * self.size as i32) as f32,
                0.,
                (pos.y * self.size as i32) as f32,
            ),
            ..Default::default()
        }
    }
}
