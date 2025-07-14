use std::marker::PhantomData;

use crate::prelude::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct InitService<T, D, E>(pub ServiceHandle<T, D, E>)
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError;
#[derive(Event)]
pub struct DisableService<T, D, E>(pub ServiceHandle<T, D, E>)
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError;
#[derive(Event)]
pub struct EnableService<T, D, E>(pub ServiceHandle<T, D, E>)
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError;
#[derive(Event)]
pub struct FailService<T, D, E>(pub ServiceHandle<T, D, E>, pub E)
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError;

#[derive(Event)]
pub struct ServiceStateChange<T, E>
where
    T: ServiceLabel,
    E: ServiceError,
{
    pub new_state: ServiceState<E>,
    pub old_state: ServiceState<E>,
    label: PhantomData<T>,
}
impl<T, E> ServiceStateChange<T, E>
where
    T: ServiceLabel,
    E: ServiceError,
{
    pub fn new(old_state: ServiceState<E>, new_state: ServiceState<E>) -> Self {
        Self {
            old_state,
            new_state,
            label: PhantomData,
        }
    }
}

#[derive(Event)]
pub struct EnterServiceState<T, E>
where
    T: ServiceLabel,
    E: ServiceError,
{
    pub new_state: ServiceState<E>,
    label: PhantomData<T>,
}
impl<T, E> EnterServiceState<T, E>
where
    T: ServiceLabel,
    E: ServiceError,
{
    pub fn new(new_state: ServiceState<E>) -> Self {
        Self {
            new_state,
            label: PhantomData,
        }
    }
}

#[derive(Event)]
pub struct ExitServiceState<T, E>
where
    T: ServiceLabel,
    E: ServiceError,
{
    pub old_state: ServiceState<E>,
    label: PhantomData<T>,
}
impl<T, E> ExitServiceState<T, E>
where
    T: ServiceLabel,
    E: ServiceError,
{
    pub fn new(old_state: ServiceState<E>) -> Self {
        Self {
            old_state,
            label: PhantomData,
        }
    }
}
