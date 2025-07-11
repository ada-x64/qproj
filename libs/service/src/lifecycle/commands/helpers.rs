use crate::{
    data::*,
    lifecycle::{
        EnterServiceState, ExitServiceState, ServiceStateChange,
        data::ServiceHooks,
    },
};
use bevy::prelude::*;

pub(crate) fn handle_error<T: ServiceName, D: ServiceData, E: ServiceError>(
    service: Entity,
    world: &mut World,
    error: E,
) {
    debug!("handle_error");
    let hook = get_hooks::<T, D, E>(world, service).on_failure;
    let id = world.register_system(hook);
    world.run_system_with(id, &error).unwrap();
    world.unregister_system(id).unwrap();
    set_state::<T, D, E>(world, service, ServiceState::Failed(error));
}
pub(crate) fn run_hook<T, D, E, F, O, M>(
    service: Entity,
    world: &mut World,
    hook: F,
) -> Result<O, ()>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
    F: IntoSystem<(), Result<O, E>, M> + 'static,
    O: 'static,
{
    let id = world.register_system(hook);
    let res = world.run_system(id).unwrap();
    world.unregister_system(id).unwrap();
    match res {
        Ok(val) => Ok(val),
        Err(error) => {
            handle_error::<T, D, E>(service, world, error);
            Err(())
        }
    }
}
pub(crate) fn set_enabled<T: ServiceName, D: ServiceData, E: ServiceError>(
    service: Entity,
    enabled: bool,
    world: &mut World,
) -> Result<(), ()> {
    let hook = if enabled {
        get_hooks::<T, D, E>(world, service).on_enable
    } else {
        get_hooks::<T, D, E>(world, service).on_disable
    };
    let res = run_hook::<T, D, E, _, _, _>(service, world, hook);
    if res.is_err() {
        return Err(());
    }
    let state = if enabled {
        ServiceState::Enabled
    } else {
        ServiceState::Disabled
    };
    set_state::<T, D, E>(world, service, state);
    Ok(())
}
pub(crate) fn set_state<T, D, E>(
    world: &mut World,
    service: Entity,
    state: ServiceState<E>,
) where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    let mut service = world
        .query::<&mut Service<T, D, E>>()
        .get_mut(world, service)
        .unwrap_or_else(|_| panic!("No service for entity id {service:?}"));
    let name = service.name.clone();
    let old_state = service.state.clone();
    service.state = state.clone();
    world.trigger(ServiceStateChange::<T, E>::new(
        name.clone(),
        old_state.clone(),
        state.clone(),
    ));
    world.trigger(EnterServiceState::<T, E>::new(name.clone(), state));
    world.trigger(ExitServiceState::<T, E>::new(name, old_state));
}

pub(crate) fn get_hooks<T, D, E>(
    world: &mut World,
    id: Entity,
) -> &ServiceHooks<E>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    &world
        .query::<&Service<T, D, E>>()
        .get(world, id)
        .unwrap_or_else(|_| panic!("No service for entity id {id:?}"))
        .hooks
}

#[derive(Deref)]
pub(crate) struct CommandInput<T, D, E>(
    #[deref] Entity,
    PhantomService<T, D, E>,
)
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError;
impl<T, D, E> CommandInput<T, D, E>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    pub(crate) fn new(id: Entity) -> Self {
        Self(id, PhantomService::default())
    }
}
