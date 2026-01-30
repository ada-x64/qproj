#![feature(register_tool)]
#![register_tool(bevy)]
#![allow(bevy::panicking_methods)]
#![doc = include_str!("./doc.md")]

mod data;
mod not;
mod plugin;
mod scope;
mod systems;
mod trait_impl;

pub mod prelude {
    pub use super::data::*;
    pub use super::not::*;
    pub use super::scope::*;
    pub(crate) use super::systems::*;
    pub use super::trait_impl::*;
    pub(crate) use bevy::prelude::*;
    pub(crate) use std::marker::PhantomData;
    pub(crate) use tiny_bail::prelude::*;
}
