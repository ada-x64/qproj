use bevy::{
    asset::RenderAssetUsages,
    color::{Color, ColorToComponents},
    math::{Vec2, Vec3},
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use itertools::Itertools;

/// size is width or length
pub fn gen_strip(size: usize) -> Vec<u32> {
    let size = size as u32;
    let mut strip = Vec::new();
    for row in 0..size - 1 {
        if row > 0 {
            strip.push(row * size);
        }
        for col in 0..size {
            strip.push(row * size + col);
            strip.push(((row + 1) * size) + col);
        }
        if row < size - 2 {
            strip.push((row + 1) * size + (size - 1));
        }
    }
    strip
}
#[test]
fn test_stip() {
    let strip = gen_strip(4);
    //  0  1  2  3
    //  4  5  6  7
    //  8  9 10 11
    // 12 13 14 15
    #[rustfmt::skip]
    let correct = vec![
           0,  4, 1,  5,  2,  6,  3,  7,  7,
        4, 4,  8, 5,  9,  6, 10,  7, 11, 11,
        8, 8, 12, 9, 13, 10, 14, 11, 15
        ];
    // 0 4 1, 1 5 2, ...
    assert_eq!(strip, correct);
}

// when piecing together chunks we'll need to take the surrounding
// chunks and set their edge normals and positions so that they're continuous
pub fn gen_normals(positions: &[Vec3]) -> Vec<Vec3> {
    positions
        .iter()
        .chunks(3)
        .into_iter()
        .flat_map(|chunk| {
            let vec = chunk.into_iter().copied().collect_vec();
            if vec.len() < 3 {
                return vec;
            }
            let [v1, v2, v3] = vec[..] else {
                unreachable!()
            };
            let face_normal = (v2 - v1).cross(v3 - v1).normalize();
            vec![
                (v1 + face_normal).normalize(),
                (v2 + face_normal).normalize(),
                (v3 + face_normal).normalize(),
            ]
        })
        .collect()
}

/// Size is width or length
pub fn gen_positions(size: usize) -> Vec<Vec3> {
    (0..size * size)
        .map(|idx| {
            Vec3::new((idx % size) as f32, rand::random(), (idx / size) as f32)
        })
        .collect_vec()
}
#[test]
fn test_size() {
    let positions = gen_positions(256);
    let normals = gen_normals(&positions);
    assert_eq!(positions.len(), normals.len());
}

pub fn gen_uvs(size: usize) -> Vec<Vec2> {
    // this will need to change based on the texture mapping
    (0..size * size)
        .map(|idx| {
            let size = size as f32;
            let idx = idx as f32;
            Vec2::new((idx % size) / size, (idx / size) / size)
        })
        .collect_vec()
}
