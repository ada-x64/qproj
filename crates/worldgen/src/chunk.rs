//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    generator::{ChunkGenerationData, Vec2i32},
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

#[derive(Default, Component)]
pub struct Chunk {
    pub pos: Vec2i32,
    pub size: usize,
    pub cells: Vec<Cell>,
}

impl Chunk {
    pub fn new(gen_data: ChunkGenerationData, pos: Vec2i32) -> Self {
        // accomodate for gap by adding +2
        // creates overlap but worth it for consistency
        let size = gen_data.size + 2;
        let noise = gen_data.expr.noise();
        let cells = (0..(usize::pow(size, 2)))
            .map(|idx| {
                let x = (idx % size) as i32;
                let y = (idx / size) as i32;
                let px = x + pos.x * gen_data.size as i32;
                let py = y + pos.y * gen_data.size as i32;
                let scale = gen_data.scale;
                let point = [px as f64 * scale, py as f64 * scale, 0.];
                let elevation = noise.get(point) * gen_data.max_elevation;
                Cell {
                    elevation,
                    ..Default::default()
                }
            })
            .collect_vec();
        Self { pos, cells, size }
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
