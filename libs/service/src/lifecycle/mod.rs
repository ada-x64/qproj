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

macro_rules! f_def {
    ($name:ident) => {
        $crate::paste::paste! {
            fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E>)
                where
                    T: ServiceName,
                    D: ServiceData,
                    E: ServiceError;
        }
    };
}
macro_rules! f_impl {
    ($name:ident) => {
        $crate::paste::paste! {
            fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E>)
                where
                    T: ServiceName,
                    D: ServiceData,
                    E: ServiceError,
            {
                self.queue([<$name:camel Service>]::<T, D, E>::new(handle));
            }
        }
    };
}

pub trait ServiceLifecycleCommands {
    f_def! {Init}
    f_def! {Enable}
    f_def! {Disable}
    fn fail_service<T, D, E>(
        &mut self,
        handle: ServiceHandle<T, D, E>,
        error: E,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError;
}
impl<'w, 's> ServiceLifecycleCommands for Commands<'w, 's> {
    f_impl!(Init);
    f_impl!(Enable);
    f_impl!(Disable);
    fn fail_service<T, D, E>(
        &mut self,
        handle: ServiceHandle<T, D, E>,
        error: E,
    ) where
        T: ServiceName,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(FailService::<T, D, E>::new(handle, error));
    }
}
