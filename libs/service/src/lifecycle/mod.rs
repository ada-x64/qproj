use crate::{
    data::*,
    lifecycle::commands::{DisableService, EnableService, InitService},
};
use bevy::prelude::*;

mod data;
pub use data::*;
mod commands;

pub trait ServiceLifecycleCommands {
    fn init_service<T, D, E>(&mut self, entity: Entity)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
    fn enable_service<T, D, E>(&mut self, entity: Entity)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
    fn disable_service<T, D, E>(&mut self, entity: Entity)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
}
impl<'w, 's> ServiceLifecycleCommands for Commands<'w, 's> {
    fn init_service<T, D, E>(&mut self, entity: Entity)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(InitService::<T, D, E>::new(entity));
    }
    fn enable_service<T, D, E>(&mut self, entity: Entity)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(EnableService::<T, D, E>::new(entity));
    }
    fn disable_service<T, D, E>(&mut self, entity: Entity)
    where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(DisableService::<T, D, E>::new(entity));
    }
}
