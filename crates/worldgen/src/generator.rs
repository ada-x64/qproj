//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
#[cfg(feature = "inspector")]
use bevy_inspector_egui::prelude::*;

use crate::expr::Expr;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub struct Vec2i32 {
    pub x: i32,
    pub y: i32,
}
impl Vec2i32 {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
impl From<Vec2> for Vec2i32 {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}
impl std::fmt::Display for Vec2i32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {})", self.x, self.y))
    }
}

#[derive(Debug, Reflect, Resource, Clone)]
#[cfg_attr(feature = "inspector", derive(InspectorOptions))]
#[reflect(Resource)]
#[cfg_attr(feature = "inspector", reflect(InspectorOptions))]
pub struct ChunkGenerator {
    /// The length and width of the chunk
    #[cfg_attr(feature = "inspector", inspector(min = 1, max = 128))]
    pub size: usize,
    /// This number is squared to determine the number of chunks spawned around the player.
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
    /// A noise_expr Expr which generates the terrain.
    pub expr: Option<Handle<Expr>>,

    /// How quickly the LOD decreases.
    /// Formula: `size / lod_scale^(dist - lod_cutoff)`
    pub lod_scale: f32,
    /// How many chunks from the player character the LOD begins to decay.
    /// Formula: `size / lod_scale^(dist - lod_cutoff)`
    pub lod_cutoff: i32,

    pub default_material: Option<Handle<StandardMaterial>>,
    pub terrain_entt: Option<Entity>,
    pub current_chunk: Option<Vec2i32>,
}
impl Default for ChunkGenerator {
    fn default() -> Self {
        Self {
            default_material: None,
            expr: None,
            terrain_entt: None,
            max_elevation: 100.,
            active_radius: 4,
            scaling_factor: 0.001,
            size: 64,
            seed: 0,
            current_chunk: None,
            lod_scale: 1.5,
            lod_cutoff: 2,
        }
    }
}
impl ChunkGenerator {
    pub fn get_data(
        &self,
        exprs: &Res<Assets<Expr>>,
    ) -> Result<ChunkGenerationData, &'static str> {
        let id = self.expr.as_ref().ok_or("Noise expr not set")?;
        let expr = exprs.get(id).ok_or("Noise expr not initiated")?;
        Ok(ChunkGenerationData {
            max_elevation: self.max_elevation,
            scale: self.scaling_factor,
            size: self.size,
            expr: expr.clone(),
        })
    }
    pub fn world_pos_to_chunk_pos(&self, pos: Vec2) -> Vec2i32 {
        let x = (pos.x.round() / self.size as f32) as i32;
        let y = (pos.y.round() / self.size as f32) as i32;
        Vec2i32::new(x, y)
    }
    /// Calculates the number of vertices given the distance from the player.
    pub fn get_num_verts(&self, dist: i32) -> usize {
        if dist > self.lod_cutoff {
            self.size / (2 * (dist - self.lod_cutoff)) as usize
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
    pub fn get_transform(&self, pos: Vec2i32) -> Transform {
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
