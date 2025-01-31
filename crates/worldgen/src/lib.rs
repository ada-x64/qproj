use bevy::{
    asset::RenderAssetUsages,
    math::Vec3,
    render::{
        mesh::{Indices, Mesh, PrimitiveTopology},
        render_resource::ShaderType,
    },
};
use itertools::Itertools;

fn gen_strip(size: u32) -> Vec<u32> {
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

fn gen_positions(size: u32) -> Vec<Vec3> {
    (0..size * size)
        .map(|idx| Vec3::new((idx % size) as f32, rand::random(), (idx / size) as f32))
        .collect_vec()
}

// when piecing together chunks we'll need to take the surrounding
// chunks and set their edge normals and positions so that they're continuous
fn gen_normals(positions: &[Vec3]) -> Vec<Vec3> {
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

#[test]
fn test_size() {
    let positions = gen_positions(256);
    let normals = gen_normals(&positions);
    assert_eq!(positions.len(), normals.len());
}

pub fn gen_mesh(size: u32) -> Mesh {
    let positions = gen_positions(size);
    println!("positions size: {:?}", positions.size());
    let normals = gen_normals(&positions);
    println!("normals size: {:?}", normals.size());
    let uvs = (0..size * size)
        .map(|idx| [(idx % 64) as f32 / 255., (idx / 64) as f32 / 255.])
        .collect_vec();
    println!("uvs size: {:?}", uvs.size());

    let strip = gen_strip(size);
    println!("strip size: {:?}", strip.size());

    // should also generate normals and uvs
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(strip));

    mesh
}
