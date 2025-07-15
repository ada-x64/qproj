// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod app;
mod data;
mod graph;
mod lifecycle;
mod service;
mod service_macro;
mod spec;

pub mod prelude {
    pub use crate::{
        app::*,
        data::*,
        lifecycle::{commands::*, events::*, hooks::*, run_conditions::*},
        service,
        service::*,
        spec::*,
    };
    pub use q_service_macros::*;
}
pub use paste;
