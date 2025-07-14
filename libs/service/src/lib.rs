// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod app;
mod data;
mod graph;
mod helpers;
mod lifecycle;
mod service;
mod spec;

pub mod prelude {
    pub use crate::{
        app::*,
        data::*,
        helpers::*,
        lifecycle::{commands::*, events::*, hooks::*},
        service,
        service::*,
        spec::*,
    };
    pub use q_service_macros::*;
}
pub use paste;
