use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use itertools::Itertools;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};

use crate::mesh::{gen_list, gen_normals, gen_strip, gen_uvs};

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

#[derive(Copy, Clone, Debug)]
pub struct ChunkGenerator {
    /// The length and width of the chunk
    pub size: usize,
    /// Maximum elevation
    pub max_elevation: f64,
    /// The seed for the perlin noise generator
    pub seed: u32,
    /// Perlin noise scaling factor must be a float other than 1.
    /// The larger the number, the smoother the terrain.
    pub perlin_scaling_factor: f64,
    /// The primary noise generator.
    pub perlin: Perlin,
}
impl Default for ChunkGenerator {
    fn default() -> Self {
        Self {
            size: 128,
            max_elevation: 10.,
            seed: Perlin::DEFAULT_SEED,
            perlin_scaling_factor: 100.,
            perlin: Perlin::default(),
        }
    }
}
impl ChunkGenerator {
    pub fn new(
        size: usize,
        max_elevation: f64,
        seed: u32,
        perlin_scaling_factor: f64,
    ) -> Self {
        let perlin = noise::Perlin::new(seed);
        Self {
            size,
            max_elevation,
            seed,
            perlin_scaling_factor,
            perlin,
        }
    }
    pub fn generate(&self, x: i32, y: i32) -> Chunk {
        Chunk::new(self, x, y)
    }
    fn get_elevation(&self, x: i32, y: i32) -> f64 {
        let px = (x as f64) / self.perlin_scaling_factor;
        let py = (y as f64) / self.perlin_scaling_factor;
        self.perlin.get([px, py]) * self.max_elevation
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
                let elevation = generator.get_elevation(
                    x + x_offset * generator.size as i32,
                    y + y_offset * generator.size as i32,
                );
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
        let hue = rand::random::<f32>() * 360.;
        let positions = self.positions();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_COLOR,
            (0..positions.len())
                .map(|_| Color::hsl(hue, 1., 0.5).to_linear().to_vec4())
                .collect_vec(),
        )
        .with_inserted_indices(Indices::U32(gen_list(self.size)))
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, gen_uvs(self.size))
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            gen_normals(&positions),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        // .with_generated_tangents()
    }
}

#[test]
fn test_chunks() {
    let chunk_size = 2;
    let generator = ChunkGenerator {
        size: chunk_size,
        ..Default::default()
    };
    let world_size = 3;
    let chunk_vertices = (0..world_size * world_size)
        .map(|idx| {
            let x = idx % world_size;
            let y = idx / world_size;
            generator.generate(x, y).positions()
        })
        .collect_vec();

    // check for overlaps and gaps
    // want to check that the perimeter lines up with other chunks
    // this is the same as taking the first and last rows and columns
    // which is to say, the first and last elements of the array
    // and the first and last elements of the inner elements of the array

    (0..world_size * world_size).for_each(|idx| {
        // get the surrounding 8 chunks and check perimeters
        let at = |x: i32, y: i32| {
            ((x + idx) % world_size) * world_size + ((y + idx) / world_size)
        };
        let this = chunk_vertices.get(idx as usize).unwrap();
        [-1, 1].into_iter().for_each(|pos| {
            let doit = |horiz: bool| {
                let other_idx = if horiz { at(pos, 0) } else { at(0, pos) };
                if other_idx < 0 || other_idx > world_size {
                    return;
                }
                let other = chunk_vertices.get(other_idx as usize).unwrap();
                let size = chunk_size + 2;
                let get_perim = |vec: &Vec<_>, target| {
                    vec.iter()
                        .copied()
                        .enumerate()
                        .filter_map(|(idx, vec)| {
                            if horiz {
                                (idx % size == target).then_some(vec)
                            } else {
                                (idx / size == target).then_some(vec)
                            }
                        })
                        .collect_vec()
                };
                let maybe_perim = if pos == -1 {
                    Some((get_perim(other, size - 1), get_perim(this, 0)))
                } else if pos == 1 {
                    Some((get_perim(other, 0), get_perim(this, size - 1)))
                } else {
                    None
                };

                if let Some((other, this)) = maybe_perim {
                    other.iter().zip(&this).for_each(|(other, this)| {
                        // TODO: Assert that the worldspace coords are the same.
                        // then, assert that the heights are the same.
                    });
                    // let mut did_test = false;
                    // other.iter().for_each(|other| {
                    //     this.iter().for_each(|this| {
                    //         // get worldspace coords
                    //         println!("{:?} ?= {:?}", other.y, this.y);
                    //         // if other.xz() == this.xz() {
                    //         // did_test = true;
                    //         assert_eq!(other, this);
                    //         // }
                    //     })
                    // });
                    // assert!(did_test, "pos={pos},\nhoriz={horiz}\nthis={this:?}\nother={other:?}");
                }
            };
            doit(true);
            doit(false);
        });
    });

    // let perimeters = chunk_vertices
    //     .iter()
    //     .map(|chunk| chunk.positions())
    //     .map(|positions| {
    //         let size = world_size + 2;
    //         positions
    //             .into_iter()
    //             .enumerate()
    //             .filter_map(|(idx, vec)| {
    //                 let x = idx % size;
    //                 let y = idx / size;
    //                 (x == 0 || x == size || y == 0 || y == size).then_some(vec)
    //             })
    //             .collect_vec()
    //     })
    //     .collect_vec();

    // check that chunks are arranged in a square
}
// #[test]
// fn test_perlin() {
//     let perlin = noise::Perlin::default();
//     (0..16).for_each(|i| {
//         let x = (i % 4) as f64 + rand::random::<f64>();
//         let y = (i / 4) as f64 + rand::random::<f64>();
//         let val = perlin.get([x, y]);
//         println!("({x}, {y}) => {val}")
//     });
// }
