#![doc = include_str!("./doc.md")]

use bevy::app::HierarchyPropagatePlugin;

use crate::prelude::*;

mod data;
mod scope;
mod systems;
mod trait_impl;

pub mod prelude {
    pub use super::data::*;
    pub use super::scope::*;
    pub(crate) use super::systems::*;
    pub use super::trait_impl::*;
    pub use bevy_asset_loader::prelude::*;
}

pub fn plugin(app: &mut App) {
    app.add_plugins((
        HierarchyPropagatePlugin::<Persistent>::new(PostUpdate),
        HierarchyPropagatePlugin::<ScreenScoped>::new(PostUpdate),
    ));
    app.add_plugins(systems::plugin);
}
