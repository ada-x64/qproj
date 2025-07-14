use crate::prelude::*;
use bevy::prelude::*;

/// Creates aliases for the commonly used service types.
/// Parameters:
/// $t: The service's name.
/// $d: The data type.
/// $e: The error type.
#[macro_export]
macro_rules! service {
    ($t:ty, $d:ty, $e:ty) => {
        $crate::paste::paste! {
            mod [<$t:lower:snake _alias_impl>] {
                use super::*;
                use $crate::prelude::*;
                use std::marker::PhantomData;
                #[derive(ServiceLabel, PartialEq, Eq, Debug, Copy, Clone, Hash)]
                pub struct [<$t Label>];
                pub type [<$t Spec>]= ServiceSpec<[<$t Label>], $d, $e>;
                pub type [<$t>] = Service<[<$t Label>], $d, $e>;
                pub type [<$t Hooks>] = ServiceHooks<$e>;
                pub type [<$t StateChange>] = ServiceStateChange<[<$t Label>], $e>;
                pub type [<Enter $t State>] = EnterServiceState<[<$t Label>], $e>;
                pub type [<Exit $t State>] = ExitServiceState<[<$t Label>], $e>;
                pub const [<$t:snake:upper >]: ServiceHandle<[<$t Label>], $d, $e> = ServiceHandle::<[<$t Label>], $d, $e>::const_default();
                pub const [<$t:snake:upper _SPEC>]: ServiceSpec<[<$t Label>], $d, $e> = ServiceSpec::<[<$t Label>], $d, $e>::const_default();
            }
            pub use [< $t:lower _alias_impl >]::*;
        }
    };
}

/// Run condition which checks if the given service has the given state.
pub fn service_has_state<T, D, E>(
    _handle: ServiceHandle<T, D, E>,
    state: ServiceState<E>,
) -> impl Condition<()>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |service: Res<Service<T, D, E>>| {
        service.state == state
    })
}

macro_rules! run_conditions {
    ($($state:ident),*) => {
        $crate::paste::paste! {
            $(
                /// Run condition
                pub fn [<service_ $state:snake:lower>]<T, D, E>(
                    _handle: ServiceHandle<T,D,E>,
                ) -> impl Condition<()>
                where
                    T: ServiceLabel,
                    D: ServiceData,
                    E: ServiceError,
                {
                    IntoSystem::into_system(
                        move |service: Res<Service<T, D, E>>| {
                            matches!(service.state, ServiceState::[<$state:camel>])
                        },
                    )
                }
            )*
        }
    };
}

run_conditions!(Uninitialized, Initializing, Enabled, Disabled);

/// Run condition
pub fn service_failed<T, D, E>(
    _handle: ServiceHandle<T, D, E>,
) -> impl Condition<()>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |service: Res<Service<T, D, E>>| {
        matches!(service.state, ServiceState::Failed(_))
    })
}

/// Run condition
pub fn service_failed_with_error<T, D, E>(
    _handle: ServiceHandle<T, D, E>,
    error: E,
) -> impl Condition<()>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |service: Res<Service<T, D, E>>| {
        if let ServiceState::Failed(e) = &service.state {
            *e == error
        } else {
            false
        }
    })
}
