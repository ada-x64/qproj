use crate::prelude::*;
use bevy::prelude::*;

macro_rules! f_def {
    ($name:ident) => {
        $crate::paste::paste! {
            fn [<$name:snake:lower _service>]<T, D, E>(&mut self, handle: ServiceHandle<T, D, E>)
                where
                    T: ServiceLabel,
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
                    T: ServiceLabel,
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
        T: ServiceLabel,
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
        T: ServiceLabel,
        D: ServiceData,
        E: ServiceError,
    {
        self.queue(FailService::<T, D, E>::new(handle, error));
    }
}

macro_rules! commands {
    ($(( $name:ident, $fn:ident ),)+) => {
        $(
        #[derive(Deref)]
        pub(crate) struct $name<T, D, E>(ServiceHandle<T, D, E>)
        where
            T: ServiceLabel,
            D: ServiceData,
            E: ServiceError;
        impl<T, D, E> $name<T, D, E>
        where
            T: ServiceLabel,
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
            T: ServiceLabel,
            D: ServiceData,
            E: ServiceError,
        {
            fn apply(self, world: &mut World) {
                // TODO: This is causing an error during testing.
                // Can't call a hook within a hook, because the
                // resource is being taken away!
                // Would be better to use an observer / events, probably.
                world.resource_scope(
                    |world, mut service: Mut<Service<T, D, E>>| {
                        let _ = service.$fn(world);
                    },
                )
            }
        }
    };
}

commands!(
    (InitService, on_init),
    (EnableService, on_enable),
    (DisableService, on_disable),
);
pub(crate) struct FailService<T, D, E>(ServiceHandle<T, D, E>, E)
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError;
impl<T, D, E> FailService<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    pub fn new(handle: ServiceHandle<T, D, E>, error: E) -> Self {
        Self(handle, error)
    }
}
impl<T, D, E> Command for FailService<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn apply(self, world: &mut World) {
        world.resource_scope(|world, mut service: Mut<Service<T, D, E>>| {
            service.on_failure(world, self.1);
        })
    }
}
