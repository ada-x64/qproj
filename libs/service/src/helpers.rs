use crate::prelude::*;
use bevy::prelude::*;

/// Creates aliases for the commonly used service types.
/// Parameters:
/// $name: The name of the service. We'll append 'Service' for you.
/// $t: The label type.
/// $d: The data type.
/// $e: The error type.
#[macro_export]
macro_rules! service {
    ($name:ident, $t:ty, $d:ty, $e:ty) => {
        $crate::paste::paste! {
            mod [<$name:lower:snake _alias_impl>] {
                use super::*;
                use $crate::prelude::*;
                use std::marker::PhantomData;
                pub type [<$name ServiceSpec>]= ServiceSpec<$t, $d, $e>;
                pub type [<$name Service>] = Service<$t, $d, $e>;
                pub type [<$name ServiceHooks>] = ServiceHooks<$e>;
                pub type [<$name ServiceStateChange>] = ServiceStateChange<$t, $e>;
                pub type [<Enter $name ServiceState>] = EnterServiceState<$t, $e>;
                pub type [<Exit $name ServiceState>] = ExitServiceState<$t, $e>;
                pub const [<$name:snake:upper _SERVICE>]: ServiceHandle<$t, $d, $e> = ServiceHandle::<$t, $d, $e>::const_default();
                pub const [<$name:snake:upper _SERVICE_SPEC>]: ServiceSpec<$t, $d, $e> = ServiceSpec::<$t, $d, $e>::const_default();
            }
            pub use [< $name:lower _alias_impl >]::*;
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
