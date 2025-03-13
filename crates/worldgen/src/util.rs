//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{ecs::system::SystemId, prelude::*};

/// Takes a size, squares it, and returns a map with (x,y) coordinates.
pub fn iter_xy(size: i32) -> impl Iterator<Item = (i32, i32)> {
    (0..size * size).map(move |idx| ((idx % size), (idx / size)))
}

#[derive(Default, Component)]
pub struct Terrain;

#[derive(Component)]
pub struct Callback(pub SystemId);

#[derive(Component)]
pub struct CallbackTriggered;
