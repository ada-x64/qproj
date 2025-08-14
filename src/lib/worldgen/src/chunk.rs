// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use itertools::Itertools;
use meshopt::SimplifyOptions;
use serde::{Deserialize, Serialize};
use std::mem::offset_of;
use thiserror::Error;

use crate::{
    generator::ChunkGenerationData,
    mesh::{gen_list, gen_uvs},
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
    pub pos: IVec2,
    pub size: usize,
    pub cells: Vec<Cell>,
    pub lod: f32,
}

impl Chunk {
    pub fn new(gen_data: ChunkGenerationData, pos: IVec2, lod: f32) -> Self {
        // accomodate for gap by adding +2
        // creates overlap but worth it for consistency
        let size = gen_data.size + 1;
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
        Self {
            pos,
            cells,
            size,
            lod,
        }
    }
    pub fn verts(&self) -> Vec<meshopt::Vertex> {
        self.cells
            .iter()
            .enumerate()
            .map(|(idx, c)| {
                let x = idx % self.size;
                let y = idx / self.size;
                // Vec3(x as f32, c.elevation as f32, y as f32)
                meshopt::Vertex {
                    p: [x as f32, c.elevation as f32, y as f32],
                    ..Default::default()
                }
            })
            .collect_vec()
    }
    pub fn to_mesh(&self) -> Mesh {
        let verts = self.verts();
        let mut indices = gen_list(self.size);

        let adapter = meshopt::VertexDataAdapter::new(
            meshopt::typed_to_bytes(&verts),
            std::mem::size_of::<meshopt::Vertex>(),
            offset_of!(meshopt::Vertex, p),
        )
        .unwrap();
        let verts = meshopt::optimize_vertex_fetch(&mut indices, &verts);
        let indices = meshopt::simplify(
            &indices,
            &adapter,
            (verts.len() as f32 * self.lod).round() as usize,
            0.001,
            SimplifyOptions::LockBorder,
            None,
        );
        let verts = verts
            .into_iter()
            .map(|v| Vec3::from_array(v.p))
            .collect_vec();

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, gen_uvs(self.size))
        .with_computed_smooth_normals()
        .with_generated_tangents()
        .expect("could not generate tangets")
    }
}

#[test]
fn f() {
    Chunk::new(
        ChunkGenerationData {
            expr: crate::expr::Expr(noise_gui::Expr::Value(noise_gui::Variable::Anonymous(13))),
            size: 32,
            scale: 0.001,
            max_elevation: 100.,
        },
        IVec2::new(0, 0),
        1.,
    )
    .to_mesh();
}
