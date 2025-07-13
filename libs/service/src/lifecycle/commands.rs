mod helpers;
use helpers::*;

use crate::data::*;
use bevy::prelude::*;

macro_rules! commands {
    ($(( $name:ident, $fn:ident ),)+) => {
        $(
        #[derive(Deref)]
        pub(crate) struct $name<T, D, E>(ServiceHandle<T, D, E>)
        where
            T: ServiceName,
            D: ServiceData,
            E: ServiceError;
        impl<T, D, E> $name<T, D, E>
        where
            T: ServiceName,
            D: ServiceData,
            E: ServiceError,
        {
            pub fn new(handle: ServiceHandle<T,D,E>) -> Self {
                Self(handle)
            }
        }

        impl_command!($name, $fn);
        )+
    };
}

macro_rules! impl_command {
    ($name:ident, $fn:ident) => {
        impl<T, D, E> Command for $name<T, D, E>
        where
            T: ServiceName,
            D: ServiceData,
            E: ServiceError,
        {
            fn apply(self, world: &mut World) {
                let mut entt = world.query::<(Entity, &Service<T, D, E>)>();
                let entt =
                    entt.iter(world).find(|(_, s)| &s.name == self.name());
                if entt.is_none() {
                    error!(
                        "Could not find service with name {:?}",
                        self.name()
                    );
                    return;
                }
                $fn::<T, D, E>(self, world, entt.unwrap().0);
            }
        }
    };
}

commands!(
    (InitService, init_service),
    (EnableService, enable_service),
    (DisableService, disable_service),
);

pub(crate) fn init_service<T, D, E>(
    _: InitService<T, D, E>,
    world: &mut World,
    service: Entity,
) where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    debug!("Initializing service {service:?}");
    set_state::<T, D, E>(world, service, ServiceState::Initializing);
    let hook = get_hooks::<T, D, E>(world, service).on_init;
    let _ = run_hook::<T, D, E, _, _, _>(service, world, hook)
        .and_then(|enabled| set_enabled::<T, D, E>(service, enabled, world));
}

pub(crate) fn enable_service<T, D, E>(
    _: EnableService<T, D, E>,
    world: &mut World,
    service: Entity,
) where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    debug!("Enabling service {service:?}");
    let _ = set_enabled::<T, D, E>(service, true, world);
}

pub(crate) fn disable_service<T, D, E>(
    _: DisableService<T, D, E>,
    world: &mut World,
    service: Entity,
) where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    debug!("Disabling service {service:?}");
    let _ = set_enabled::<T, D, E>(service, false, world);
}

pub(crate) struct FailService<T, D, E>(ServiceHandle<T, D, E>, E)
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError;
impl<T, D, E> FailService<T, D, E>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    pub fn new(handle: ServiceHandle<T, D, E>, error: E) -> Self {
        Self(handle, error)
    }
    pub fn name(&self) -> &T {
        self.0.name()
    }
    pub fn error(&self) -> &E {
        &self.1
    }
}
pub(crate) fn fail_service<T, D, E>(
    s: FailService<T, D, E>,
    world: &mut World,
    service: Entity,
) where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    debug!("Failing service {service:?}");
    handle_error::<T, D, E>(service, world, s.error().clone());
}
impl_command!(FailService, fail_service);
