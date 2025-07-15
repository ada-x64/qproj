//! This module is about the service lifecycle.
//!
//! Services go through a few distinct phases, as represented by the chart
//! below. ![service-lifecycle.png](TODO)
//!
//! When initializing a service, you can add hooks to each lifecycle phase.
//! For example:
//!
//! ```rust
//! use bevy::prelude::*;
//! use q_service::prelude::*;
//!
//! service!(Example, (), ExampleError)
//!
//! let app = App::new();
//! app.add_service(
//!     EXAMPLE_SERVICE_SPEC
//!         .on_init(|| info!("This can be any system."));
//! )
//! ```
//!
//! You can also react to service state changes using events...
//!
//! ```rust
//! app.add_observer(|trigger: Trigger<ExampleServiceEnabled>| {/*...*/})
//! ```
//!
//! ... and you can define run conditions

pub mod commands;
pub mod events;
pub mod hooks;
pub mod run_conditions;
