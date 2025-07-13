// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod data;
pub mod helpers;
mod lifecycle;

use data::*;
use lifecycle::*;

pub mod prelude {
    pub use crate::{
        ServiceExt,
        data::*,
        helpers::*,
        lifecycle::{
            EnterServiceState, ExitServiceState, ServiceHooks,
            ServiceLifecycleCommands, ServiceStateChange,
        },
        service,
    };
    pub use q_service_macros::*;
}
pub use paste;

use bevy::{ecs::system::RunSystemOnce, prelude::*};

/// Plugin for service management.
pub trait ServiceExt<T: ServiceName, D: ServiceData, E: ServiceError> {
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self;
}
impl<T: ServiceName, D: ServiceData, E: ServiceError> ServiceExt<T, D, E>
    for App
{
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self {
        self.add_event::<ServiceStateChange<T, E>>();
        self.add_event::<EnterServiceState<T, E>>();
        self.add_event::<ExitServiceState<T, E>>();
        let world = self.world_mut();
        world
            .run_system_once_with(add_service_inner, spec)
            .expect("Failed to add service! Spec: {spec:?}");
        self
    }
}

fn add_service_inner<T: ServiceName, D: ServiceData, E: ServiceError>(
    spec: In<ServiceSpec<T, D, E>>,
    mut commands: Commands,
    services: Query<&Service<T, D, E>>,
) {
    let spec = spec.clone();
    let found = services.iter().any(|service| service.name == spec.name);
    if found {
        warn!("Tried to add already existing service {:?}", spec.name);
        return;
    }
    let is_startup = spec.is_startup;
    let name = spec.name.clone();
    commands.spawn(Service::<T, D, E>::from_spec(spec));
    if is_startup {
        commands.init_service(ServiceHandle::<T, D, E>::new(name));
    }
}
