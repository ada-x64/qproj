use crate::data::*;
use bevy::{
    ecs::system::{
        FunctionSystem, IsExclusiveFunctionSystem, IsFunctionSystem,
        RegisteredSystemError, RunSystemOnce, SystemParam,
    },
    platform::collections::HashMap,
    prelude::*,
};
use std::{fmt::Debug, marker::PhantomData};

pub type HookError<E: ServiceError> = RegisteredSystemError<(), E>;

/// Service lifecycle hook which executes on initialization.
pub trait InitFn<E: ServiceError, M> {}
impl<T, E, M> InitFn<E, M> for T
where
    E: ServiceError,
    T: IntoSystem<(), Result<bool, E>, M> + Debug,
{
}

/// Service lifecycle hook which executes on initialization.
pub trait EnableFn<E: ServiceError, M> {}
impl<T, E, M> EnableFn<E, M> for T
where
    T: IntoSystem<(), Result<(), E>, M>,
    E: ServiceError,
{
}

/// Service lifecycle hook which executes on initialization.
#[allow(type_alias_bounds)]
pub trait FailFn<E: ServiceError, M> {}
#[allow(type_alias_bounds)]
impl<T, E, M> FailFn<E, M> for T
where
    for<'i> T: IntoSystem<InRef<'i, HookError<E>>, (), M>,
    E: ServiceError,
{
}

/// Service lifecycle hooks.
pub struct ServiceHooks<E, IM, EM, DM, FM> {
    pub on_init: Box<dyn InitFn<E, IM>>,
    pub on_enable: Box<dyn EnableFn<E, EM>>,
    pub on_disable: Box<dyn EnableFn<E, DM>>,
    pub on_failure: Box<dyn FailFn<E, FM>>,
}
impl<E, IM, EM, DM, FM> Default for ServiceHooks<E, IM, EM, DM, FM>
where
    E: ServiceError,
{
    fn default() -> Self {
        Self {
            on_init: Box::new(default_init),
            on_enable: default_enable,
            on_disable: default_enable,
            on_failure: default_fail,
            err: PhantomData::default(),
        }
    }
}
fn do_something<E: ServiceError>(world: &mut World) {
    world.run_system_once(default_init::<E>);
}
pub fn default_init<E: ServiceError>() -> Result<bool, E> {
    Ok(true)
}
pub fn default_enable<E: ServiceError>(_: &mut World) -> Result<(), E> {
    Ok(())
}
pub fn default_fail<E: ServiceError>(
    reason: InRef<HookError<E>>,
    _: &mut World,
) {
    error!("Service failed with error: {reason:#?}");
}
// impl<E: ServiceError> ServiceHooks<E> {
//     pub fn new() -> Self {
//         Self::default()
//     }
//     pub fn on_init(self, on_init: InitFn<E>) -> Self {
//         Self { on_init, ..self }
//     }
//     pub fn on_enable(self, on_enable: EnableFn<E>) -> Self {
//         Self { on_enable, ..self }
//     }
//     pub fn on_disable(self, on_disable: EnableFn<E>) -> Self {
//         Self { on_disable, ..self }
//     }
//     pub fn on_failure(self, on_failure: FailFn<E>) -> Self {
//         Self { on_failure, ..self }
//     }
// }

#[derive(Event)]
pub struct ServiceStateChange<T, E>
where
    T: ServiceName,
    E: ServiceError,
{
    pub name: T,
    pub new_state: ServiceState<E>,
    pub old_state: ServiceState<E>,
}
impl<T, E> ServiceStateChange<T, E>
where
    T: ServiceName,
    E: ServiceError,
{
    pub fn new(
        name: T,
        old_state: ServiceState<E>,
        new_state: ServiceState<E>,
    ) -> Self {
        Self {
            name,
            old_state,
            new_state,
        }
    }
}

#[derive(Event)]
pub struct EnterServiceState<T, E>
where
    T: ServiceName,
    E: ServiceError,
{
    pub name: T,
    pub new_state: ServiceState<E>,
}
impl<T, E> EnterServiceState<T, E>
where
    T: ServiceName,
    E: ServiceError,
{
    pub fn new(name: T, new_state: ServiceState<E>) -> Self {
        Self { name, new_state }
    }
}

#[derive(Event)]
pub struct ExitServiceState<T, E>
where
    T: ServiceName,
    E: ServiceError,
{
    pub name: T,
    pub old_state: ServiceState<E>,
}
impl<T, E> ExitServiceState<T, E>
where
    T: ServiceName,
    E: ServiceError,
{
    pub fn new(name: T, old_state: ServiceState<E>) -> Self {
        Self { name, old_state }
    }
}
