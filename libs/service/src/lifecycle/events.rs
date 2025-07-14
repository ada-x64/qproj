use crate::prelude::*;
use bevy::prelude::*;

macro_rules! trigger_hook_events {
    ($(($name:ident$(, $err:ident)*)$(,)?)*) => {
        $crate::paste::paste! {
            $(
                #[derive(Event)]
                pub struct [<$name Service>]<T, D, E>(pub ServiceHandle<T, D, E>, $(pub $err)*)
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError;
            )*
        }
    }
}
trigger_hook_events!((Enable), (Disable), (Init), (Fail, E));

macro_rules! state_change {
    ( $( ($name:ident, $($ss:ty)+)$(,)?)* ) => {
        $(
            #[derive(Event, Deref)]
            pub struct $name<T, D, E>(
                #[deref]
                $($ss)*,
                ServiceHandle<T,D,E>
            )
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
                pub fn new(val: $($ss)*) -> Self {
                    Self(val, ServiceHandle::const_default())
                }
                pub fn new_with_handle(handle: ServiceHandle<T,D,E>, val: $($ss)*) -> Self {
                    Self(val, handle)
                }
            }
        )*
    };
}
state_change!(
    (ServiceStateChange, (ServiceState<E>, ServiceState<E>)),
    (ExitServiceState, ServiceState<E>),
    (EnterServiceState, ServiceState<E>),
);

macro_rules! enter_state_aliases {
    ($(($name:ident$(, $err_ty:ident )*)$(,)?)*) => {
        $crate::paste::paste! {
            $(
                #[allow(dead_code, reason = "macro gen")]
                #[derive(Event)]
                pub struct [<Service $name>]<T, D, E>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError,
                {
                    _handle: ServiceHandle<T, D, E>,
                    $(err: $err_ty)*
                }
                impl<T, D, E> [<Service $name>]<T, D, E>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError,
                {
                    pub fn new(_handle: ServiceHandle<T, D, E>, $(err: $err_ty)*) -> Self {
                        Self { _handle, $(err: err as $err_ty)* }
                    }
                }
            )*
        }
    };
}

enter_state_aliases!((Enabled), (Disabled), (Initialized), (Failed, E));
