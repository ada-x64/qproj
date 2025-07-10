// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

mod data;
mod lifecycle;

pub use data::*;
pub use lifecycle::*;

use bevy::prelude::*;

/// Plugin for service management.
pub struct ServicePlugin<T: ServiceNames> {
    pub services: Vec<ServiceSpec<T>>,
}

impl<T: ServiceNames> Plugin for ServicePlugin<T> {
    fn build(&self, app: &mut App) {
        let specs = self.services.clone();
        app
            .init_resource::<ServiceDependencies<T>>()
            .init_resource::<ServiceLifecycles<T>>()
            .add_systems(
            Startup,
            move |mut commands: Commands,
                  query: Query<Entity, With<ServiceManager>>,
                  services: Query<&Service<T>>,
                  mut lifecycles: ResMut<ServiceLifecycles<T>>,
                  mut deps: ResMut<ServiceDependencies<T>>| {
                      let manager = query.single().unwrap_or_else(|_|{
                          commands.spawn(ServiceManager).id()
                      });
                specs.iter().filter(|spec| {
                    let found = services
                        .iter()
                        .any(|service| **service == spec.name);
                    if found {
                        warn!(
                            "Tried to add already existing service {:?}",
                            spec.name
                        );
                    }
                    !found
                    }).for_each(|spec| {
                        deps.insert(spec.name, spec.deps.clone());
                        lifecycles.insert(spec.name, spec.lifecycle);
                        commands.entity(manager).with_child(ServiceBundle::from(spec));
                        if spec.is_startup {
                            commands.init_service(spec.name);
                        }
                    });
            },
        );
    }
}
