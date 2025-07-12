use crate::{
    data::*,
    lifecycle::commands::{
        DisableService, EnableService, FailService, InitService,
    },
};
use bevy::prelude::*;

mod data;
pub use data::*;
mod commands;

pub trait ServiceLifecycleCommands {
    fn init_service<T, D, E>(
        &mut self,
        name: T,
        marker: ServiceMarker<T, D, E>,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
    fn enable_service<T, D, E>(
        &mut self,
        name: T,
        marker: ServiceMarker<T, D, E>,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
    fn disable_service<T, D, E>(
        &mut self,
        name: T,
        marker: ServiceMarker<T, D, E>,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
    fn fail_service<T, D, E>(
        &mut self,
        name: T,
        error: E,
        marker: ServiceMarker<T, D, E>,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
}
impl<'w, 's> ServiceLifecycleCommands for Commands<'w, 's> {
    fn init_service<T, D, E>(&mut self, name: T, _: ServiceMarker<T, D, E>)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(InitService::<T, D, E>::new(name));
    }
    fn enable_service<T, D, E>(&mut self, name: T, _: ServiceMarker<T, D, E>)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(EnableService::<T, D, E>::new(name));
    }
    fn disable_service<T, D, E>(&mut self, name: T, _: ServiceMarker<T, D, E>)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(DisableService::<T, D, E>::new(name));
    }
    fn fail_service<T, D, E>(
        &mut self,
        name: T,
        error: E,
        _: ServiceMarker<T, D, E>,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(FailService::<T, D, E>::new(name, error));
    }
}
