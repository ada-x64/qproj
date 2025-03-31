use crate::{ffi, DecodePosition, VertexDataAdapter};
use std::mem;

pub type VertexCacheStatistics = ffi::meshopt_VertexCacheStatistics;
pub type VertexFetchStatistics = ffi::meshopt_VertexFetchStatistics;
pub type OverdrawStatistics = ffi::meshopt_OverdrawStatistics;

/// Returns cache hit statistics using a simplified FIFO model.
/// Results may not match actual GPU performance.
pub fn analyze_vertex_cache(
    indices: &[u32],
    vertex_count: usize,
    cache_size: u32,
    warp_size: u32,
    prim_group_size: u32,
) -> VertexCacheStatistics {
    unsafe {
        ffi::meshopt_analyzeVertexCache(
            indices.as_ptr(),
            indices.len(),
            vertex_count,
            cache_size,
            warp_size,
            prim_group_size,
        )
    }
}

/// Returns cache hit statistics using a simplified direct mapped model.
/// Results may not match actual GPU performance.
pub fn analyze_vertex_fetch(
    indices: &[u32],
    vertex_count: usize,
    vertex_size: usize,
) -> VertexFetchStatistics {
    unsafe {
        ffi::meshopt_analyzeVertexFetch(indices.as_ptr(), indices.len(), vertex_count, vertex_size)
    }
}

/// Returns overdraw statistics using a software rasterizer.
/// Results may not match actual GPU performance.
pub fn analyze_overdraw_decoder<T: DecodePosition>(
    indices: &[u32],
    vertices: &[T],
) -> OverdrawStatistics {
    let positions = vertices
        .iter()
        .map(|vertex| vertex.decode_position())
        .collect::<Vec<[f32; 3]>>();
    unsafe {
        ffi::meshopt_analyzeOverdraw(
            indices.as_ptr(),
            indices.len(),
            positions.as_ptr().cast(),
            positions.len(),
            mem::size_of::<f32>() * 3,
        )
    }
}

/// Returns overdraw statistics using a software rasterizer.
/// Results may not match actual GPU performance.
pub fn analyze_overdraw(indices: &[u32], vertices: &VertexDataAdapter<'_>) -> OverdrawStatistics {
    unsafe {
        ffi::meshopt_analyzeOverdraw(
            indices.as_ptr(),
            indices.len(),
            vertices.pos_ptr(),
            vertices.vertex_count,
            vertices.vertex_stride,
        )
    }
}
