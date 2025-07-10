use crate::data::*;
use bevy::{platform::collections::HashMap, prelude::*};
use std::fmt::Debug;

pub type InitFn = fn(&mut World) -> Result<bool, BevyError>;
pub type EnableFn = fn(&mut World) -> Result<(), BevyError>;
pub type FailFn = fn(InRef<BevyError>, &mut World);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ServiceLifecycle {
    pub(crate) on_init: InitFn,
    pub(crate) on_enable: EnableFn,
    pub(crate) on_disable: EnableFn,
    pub(crate) on_failure: FailFn,
}
impl Default for ServiceLifecycle {
    fn default() -> Self {
        Self {
            on_init: default_init,
            on_enable: default_enable,
            on_disable: default_enable,
            on_failure: default_fail,
        }
    }
}
fn default_init(_: &mut World) -> Result<bool, BevyError> {
    Ok(true)
}
fn default_enable(_: &mut World) -> Result<(), BevyError> {
    Ok(())
}
fn default_fail(reason: InRef<BevyError>, _: &mut World) {
    error!("Failed to initialize service with error: {reason:#?}");
}
impl ServiceLifecycle {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_init(self, on_init: InitFn) -> Self {
        Self { on_init, ..self }
    }
    pub fn on_enable(self, on_enable: EnableFn) -> Self {
        Self { on_enable, ..self }
    }
    pub fn on_disable(self, on_disable: EnableFn) -> Self {
        Self { on_disable, ..self }
    }
    pub fn on_failure(self, on_failure: FailFn) -> Self {
        Self { on_failure, ..self }
    }
}

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct ServiceLifecycles<T: ServiceNames>(HashMap<T, ServiceLifecycle>);
impl<T: ServiceNames> FromWorld for ServiceLifecycles<T> {
    fn from_world(_: &mut World) -> Self {
        Self(HashMap::default())
    }
}

fn handle_error<T: ServiceNames>(
    service_name: T,
    world: &mut World,
    lifecycle_fns: &ServiceLifecycle,
    error: BevyError,
) {
    let id = world.register_system(lifecycle_fns.on_failure);
    world.run_system_with(id, &error).unwrap();
    world.unregister_system(id).unwrap();
    let mut services = world.query::<(&Service<T>, &mut ServiceState)>();
    let (_, mut state) = services
        .iter_mut(world)
        .find(|bundle| **bundle.0 == service_name)
        .expect("No service found with name {name:?}");
    **state = ServiceStatus::Failed(error);
}
macro_rules! run_hook {
    ($name:ident, $world:ident, $lifecycle_fns:ident, $hook:expr) => {{
        let id = $world.register_system($hook);
        let res = $world.run_system(id).unwrap();
        $world.unregister_system(id).unwrap();
        match res {
            Ok(val) => Ok(val),
            Err(error) => {
                handle_error($name, $world, $lifecycle_fns, error);
                Err(())
            }
        }
    }};
}
fn set_enabled<T: ServiceNames>(
    service_name: T,
    enabled: bool,
    world: &mut World,
    lifecycle_fns: &ServiceLifecycle,
) -> Result<(), ()> {
    let res = if enabled {
        run_hook!(service_name, world, lifecycle_fns, lifecycle_fns.on_enable)
    } else {
        run_hook!(service_name, world, lifecycle_fns, lifecycle_fns.on_disable)
    };
    if res.is_err() {
        return Err(());
    }
    let mut services = world.query::<(&Service<T>, &mut ServiceState)>();
    let (_, mut state) = services
        .iter_mut(world)
        .find(|bundle| **bundle.0 == service_name)
        .expect("No service found with name {name:?}");
    if enabled {
        **state = ServiceStatus::Enabled;
    } else {
        **state = ServiceStatus::Disabled;
    }
    Ok(())
}

struct InitService<T: ServiceNames>(T);
impl<T: ServiceNames> Command for InitService<T> {
    fn apply(self, world: &mut World) {
        world.resource_scope(|world, lifecycles: Mut<ServiceLifecycles<T>>| {
            let name = self.0;
            debug!("Initializing service {name:?}");
            let lifecycle_fns = lifecycles.get(&name).unwrap_or_else(|| {
                panic!("No lifecycle functions for service {name:?}");
            });

            let _ =
                run_hook!(name, world, lifecycle_fns, lifecycle_fns.on_init)
                    .and_then(|enabled| {
                        set_enabled(name, enabled, world, lifecycle_fns)
                    });
        });
    }
}

macro_rules! impl_service_command {
    ($cmd:ty, $hook:ident) => {
        impl<T: ServiceNames> Command for $cmd {
            fn apply(self, world: &mut World) {
                world.resource_scope(
                    |world, lifecycles: Mut<ServiceLifecycles<T>>| {
                        let name = self.0;
                        let lifecycle_fns = lifecycles.get(&name).expect(
                            "No lifecycle functions for service {name:?}",
                        );
                        run_hook!(
                            name,
                            world,
                            lifecycle_fns,
                            lifecycle_fns.$hook
                        )
                        .unwrap();
                    },
                );
            }
        }
    };
}
struct EnableService<T: ServiceNames>(T);
impl_service_command!(EnableService<T>, on_enable);
struct DisableService<T: ServiceNames>(T);
impl_service_command!(DisableService<T>, on_disable);

pub trait ServiceLifecycleCommands<T: ServiceNames> {
    fn init_service(&mut self, name: T);
    fn enable_service(&mut self, name: T);
    fn disable_service(&mut self, name: T);
}
impl<'w, 's, T: ServiceNames> ServiceLifecycleCommands<T> for Commands<'w, 's> {
    fn init_service(&mut self, name: T) {
        self.queue(InitService(name));
    }
    fn enable_service(&mut self, name: T) {
        self.queue(EnableService(name));
    }
    fn disable_service(&mut self, name: T) {
        self.queue(DisableService(name));
    }
}
