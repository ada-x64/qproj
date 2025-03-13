//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use derivative::Derivative;

use crate::{chunk::ChunkGenerationData, expr::Expr};

#[derive(Derivative, Resource, Clone)]
#[derivative(Debug)]
pub struct ChunkGenerator {
    /// The length and width of the chunk
    pub size: usize,
    /// Maximum elevation
    pub max_elevation: f64,
    /// The seed for the perlin noise generator
    pub seed: u32,
    /// Perlin noise scaling factor must be a float other than 1.
    /// The larger the number, the smoother the terrain.
    pub scaling_factor: f64,
    /// A noise_expr Expr which generates the terrain.
    pub expr: Option<Handle<Expr>>,
    pub default_material: Option<Handle<StandardMaterial>>,
}
impl Default for ChunkGenerator {
    fn default() -> Self {
        Self {
            default_material: None,
            expr: None,
            max_elevation: 100.,
            scaling_factor: 0.001,
            size: 32,
            seed: 0,
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
}
