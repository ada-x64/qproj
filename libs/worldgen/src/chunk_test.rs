// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

// use itertools::Itertools;

// use crate::chunk::ChunkGenerator;

// #[allow(clippy::all)]
// #[test]
// fn test_chunks() {
//     todo!();
//     let chunk_size = 2;
//     let generator = ChunkGenerator {
//         size: chunk_size,
//         ..Default::default()
//     };
//     let world_size = 3;
//     let chunk_vertices = (0..world_size * world_size)
//         .map(|idx| {
//             let x = idx % world_size;
//             let y = idx / world_size;
//             generator.generate(x, y).positions()
//         })
//         .collect_vec();

//     // check for overlaps and gaps
//     // want to check that the perimeter lines up with other chunks
//     // this is the same as taking the first and last rows and columns
//     // which is to say, the first and last elements of the array
//     // and the first and last elements of the inner elements of the array

//     (0..world_size * world_size).for_each(|idx| {
//         // get the surrounding 8 chunks and check perimeters
//         let at = |x: i32, y: i32| {
//             ((x + idx) % world_size) * world_size + ((y + idx) / world_size)
//         };
//         let this = chunk_vertices.get(idx as usize).unwrap();
//         [-1, 1].into_iter().for_each(|pos| {
//             let doit = |horiz: bool| {
//                 let other_idx = if horiz { at(pos, 0) } else { at(0, pos) };
//                 if other_idx < 0 || other_idx > world_size {
//                     return;
//                 }
//                 let other = chunk_vertices.get(other_idx as usize).unwrap();
//                 let size = chunk_size + 2;
//                 let get_perim = |vec: &Vec<_>, target| {
//                     vec.iter()
//                         .copied()
//                         .enumerate()
//                         .filter_map(|(idx, vec)| {
//                             if horiz {
//                                 (idx % size == target).then_some(vec)
//                             } else {
//                                 (idx / size == target).then_some(vec)
//                             }
//                         })
//                         .collect_vec()
//                 };
//                 let maybe_perim = if pos == -1 {
//                     Some((get_perim(other, size - 1), get_perim(this, 0)))
//                 } else if pos == 1 {
//                     Some((get_perim(other, 0), get_perim(this, size - 1)))
//                 } else {
//                     None
//                 };

//                 if let Some((other, this)) = maybe_perim {
//                     other.iter().zip(&this).for_each(|(other, this)| {
//                         // TODO: Assert that the worldspace coords are the
// same.                         // then, assert that the heights are the same.
//                     });
//                     // let mut did_test = false;
//                     // other.iter().for_each(|other| {
//                     //     this.iter().for_each(|this| {
//                     //         // get worldspace coords
//                     //         println!("{:?} ?= {:?}", other.y, this.y);
//                     //         // if other.xz() == this.xz() {
//                     //         // did_test = true;
//                     //         assert_eq!(other, this);
//                     //         // }
//                     //     })
//                     // });
//                     // assert!(did_test,
// "pos={pos},\nhoriz={horiz}\nthis={this:?}\nother={other:?}");
// }             };
//             doit(true);
//             doit(false);
//         });
//     });

//     // let perimeters = chunk_vertices
//     //     .iter()
//     //     .map(|chunk| chunk.positions())
//     //     .map(|positions| {
//     //         let size = world_size + 2;
//     //         positions
//     //             .into_iter()
//     //             .enumerate()
//     //             .filter_map(|(idx, vec)| {
//     //                 let x = idx % size;
//     //                 let y = idx / size;
//     //                 (x == 0 || x == size || y == 0 || y ==
// size).then_some(vec)     //             })
//     //             .collect_vec()
//     //     })
//     //     .collect_vec();

//     // check that chunks are arranged in a square
// }
