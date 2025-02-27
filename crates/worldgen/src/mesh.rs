use bevy::prelude::*;
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

/// 2(n-1)^2 tris, 6(n-1)^2 indices
pub fn gen_list(size: usize) -> Vec<u32> {
    let size = size as u32;
    let mut list = Vec::new();
    for row in 0..size - 1 {
        for col in 0..size - 1 {
            let top_left = row * size + col;
            let top_right = row * size + col + 1;
            let bottom_left = (row + 1) * size + col;

            list.push(top_left);
            list.push(bottom_left);
            list.push(top_right);

            let bottom_right = (row + 1) * size + col + 1;

            list.push(top_right);
            list.push(bottom_left);
            list.push(bottom_right);
        }
    }
    list
}

#[test]
fn test_list() {
    let list = gen_list(4);
    //  0  1  2  3
    //  4  5  6  7
    //  8  9 10 11
    // 12 13 14 15
    #[rustfmt::skip]
    let correct = vec![
        // Row 0, Col 0
        0, 4, 1,
        1, 4, 5,
        // Row 0, Col 1
        1, 5, 2,
        2, 5, 6,
        // Row 0, Col 2
        2, 6, 3,
        3, 6, 7,
        // Row 1, Col 0
        4, 8, 5,
        5, 8, 9,
        // Row 1, Col 1
        5, 9, 6,
        6, 9, 10,
        // Row 1, Col 2
        6, 10, 7,
        7, 10, 11,
        // Row 2, Col 0
        8, 12, 9,
        9, 12, 13,
        // Row 2, Col 1
        9, 13, 10,
        10, 13, 14,
        // Row 2, Col 2
        10, 14, 11,
        11, 14, 15
    ];

    assert_eq!(list, correct);
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
