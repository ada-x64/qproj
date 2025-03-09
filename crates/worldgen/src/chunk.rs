use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use derivative::Derivative;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    expr::{Expr, NoiseBox},
    mesh::{gen_list, gen_normals, gen_uvs},
};

#[derive(Default, Serialize, Deserialize)]
pub struct Percent(f32);
impl Percent {
    pub fn try_new(pct: f32) -> Option<Self> {
        (0. ..=1.).contains(&pct).then_some(Self(pct))
    }
}
#[derive(Default, Serialize, Deserialize)]
pub struct Soil {
    clay: Percent,
    loam: Percent,
    sand: Percent,
}
#[derive(Default, Serialize, Deserialize)]
pub struct SoilCell {
    moisture: Percent,
    soil: Soil,
}

#[derive(Default, Serialize, Deserialize)]
pub struct FoliageCell {}

#[derive(Default, Serialize, Deserialize)]
pub struct Cell {
    pub elevation: f64,
    pub soil: SoilCell,
    pub foliage: FoliageCell,
}

#[derive(Error, Debug)]
pub enum ChunkError {
    #[error("Expression asset not yet loaded.")]
    AssetNotLoaded,
}

#[derive(Derivative, Resource)]
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
    #[derivative(Debug = "ignore")]
    // pub noise_fn: BoxedNoiseFn,
    pub expr_handle: Option<Handle<Expr>>,
}
impl ChunkGenerator {
    pub fn generate(&self, noise: &NoiseBox, x: i32, y: i32) -> Chunk {
        Chunk::new(self, noise, x, y)
    }
    pub fn get_transform(&self, x: i32, y: i32) -> Transform {
        Transform {
            translation: Vec3::new(
                (x * self.size as i32) as f32,
                0.,
                (y * self.size as i32) as f32,
            ),
            ..Default::default()
        }
    }
}

/// Takes a size, squares it, and returns a map with (x,y) coordinates.
pub fn iter_xy(size: i32) -> impl Iterator<Item = (i32, i32)> {
    (0..size * size).map(move |idx| ((idx % size), (idx / size)))
}

#[derive(Default, Component)]
pub struct Terrain;

#[derive(Default, Serialize, Deserialize, Component)]
pub struct Chunk {
    pub x_offset: i32,
    pub y_offset: i32,
    pub size: usize,
    pub cells: Vec<Cell>,
}

impl Chunk {
    pub fn new(
        generator: &ChunkGenerator,
        noise: &NoiseBox,
        x_offset: i32,
        y_offset: i32,
    ) -> Self {
        // accomodate for gap by adding +2
        // creates overlap but worth it for consistency
        let size = generator.size + 2;
        let cells = (0..(usize::pow(size, 2)))
            .map(|idx| {
                let x = (idx % size) as i32;
                let y = (idx / size) as i32;
                let px = x + x_offset * generator.size as i32;
                let py = y + y_offset * generator.size as i32;
                let scale = generator.scaling_factor;
                let point = [px as f64 * scale, py as f64 * scale, 0.];
                let elevation = noise.get(point) * generator.max_elevation;
                Cell {
                    elevation,
                    ..Default::default()
                }
            })
            .collect_vec();
        Self {
            x_offset,
            y_offset,
            cells,
            size,
        }
    }
    pub fn positions(&self) -> Vec<Vec3> {
        self.cells
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                let x = idx % self.size;
                let y = idx / self.size;
                Vec3::new(x as f32, c.elevation as f32, y as f32)
            })
            .collect_vec()
    }
    pub fn to_mesh(&self) -> Mesh {
        let positions = self.positions();
        let indices = gen_list(self.size);
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            (0..positions.len())
                .map(|_| Color::hsv(0., 0., 0.75).to_linear().to_vec4())
                .collect_vec(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            gen_normals(&positions, &indices),
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, gen_uvs(self.size))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        // .with_generated_tangents()
    }
}
