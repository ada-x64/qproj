use crate::prelude::*;
use bevy::prelude::*;

/// Creates aliases for the commonly used service types.
/// Assumes that Names is an enum.
/// Examples:
/// ```rust
/// // shorthands
/// alias!(S1, S1Err);
/// alias!(S2, S2Names, S2Err);
/// // full
/// alias!(S3, S3Names, S3Data, S3Err);
///
/// // produces...
/// type ExampleSpec = ServiceSpec<ExNames, ExData, ExErr>;
/// type ExampleService = Service<ExNames, ExData, ExErr>;
/// type ExampleHooks = ServiceHooks<ExErr>;
/// type ExampleMarker = ServiceMarker<ExNames, ExData, ExData>;
/// ```
#[macro_export]
macro_rules! service {
    ($name:ident, $t:ty, $d:ty, $e:ty) => {
        $crate::paste::paste! {
            mod [<$name:lower:snake _alias_impl>] {
                use super::*;
                use $crate::prelude::*;
                use std::marker::PhantomData;
                /// Spec for the service. See [ServiceSpec].
                pub type [<$name ServiceSpec>]= ServiceSpec<$t, $d, $e>;
                /// The main service component for the service. See [Service].
                pub type [<$name Service>] = Service<$t, $d, $e>;
                /// Lifecycle hooks for the service. See [ServiceHooks].
                pub type [<$name ServiceHooks>] = ServiceHooks<$e>;
                /// Fires when service state changes. See [ServiceStateChange]
                pub type [<$name ServiceStateChange>] = ServiceStateChange<$t, $e>;
                /// Fires when service state changes. See [EnterServiceState].
                pub type [<Enter $name ServiceState>] = EnterServiceState<$t, $e>;
                /// Fires when service state changes. See [ExitServiceState].
                pub type [<Exit $name ServiceState>] = ExitServiceState<$t, $e>;
                /// A marker which uniquely points out the service. See [ServiceHandle].
                pub const [<$name:snake:upper _SERVICE>]: ServiceHandle<$t, $d, $e> = ServiceHandle::<$t, $d, $e>::new($t::$name);
                /// The spec for the given service.
                pub const [<$name:snake:upper _SERVICE_SPEC>]: ServiceSpec<$t, $d, $e> = ServiceSpec::<$t, $d, $e>::new($t::$name);
            }
            pub use [< $name:lower _alias_impl >]::*;
        }
    };
}

/// Run condition which checks if the given service has the given state.
pub fn service_has_state<T, D, E>(
    handle: ServiceHandle<T, D, E>,
    state: ServiceState<E>,
) -> impl Condition<()>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |services: Query<&Service<T, D, E>>| {
        services
            .iter()
            .find(|s| s.name == *handle.name())
            .map(|s| s.state.eq(&state))
            .unwrap_or_default()
    })
}

macro_rules! run_condition {
    ($state:ident) => {
        $crate::paste::paste! {
            /// Run condition
            pub fn [<service_ $state:snake:lower>]<T, D, E>(
                handle: ServiceHandle<T,D,E>,
            ) -> impl Condition<()>
            where
                T: ServiceName,
                D: ServiceData,
                E: ServiceError,
            {
                IntoSystem::into_system(
                    move |services: Query<&Service<T, D, E>>| {
                        services
                            .iter()
                            .find(|s| s.name == *handle.name())
                            .map(|s| matches!(s.state, ServiceState::[<$state:camel>]))
                            .unwrap_or_default()
                    },
                )
            }
        }
    };
}

run_condition!(Uninitialized);
run_condition!(Initializing);
run_condition!(Enabled);
run_condition!(Disabled);

/// Run condition
pub fn service_failed<T, D, E>(
    handle: ServiceHandle<T, D, E>,
) -> impl Condition<()>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |services: Query<&Service<T, D, E>>| {
        services
            .iter()
            .find(|s| s.name == *handle.name())
            .map(|s| matches!(s.state, ServiceState::Failed(_)))
            .unwrap_or_default()
    })
}

/// Run condition
pub fn service_failed_with_error<T, D, E>(
    handle: ServiceHandle<T, D, E>,
    error: E,
) -> impl Condition<()>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |services: Query<&Service<T, D, E>>| {
        services
            .iter()
            .find(|s| s.name == *handle.name())
            .map(|s| {
                if let ServiceState::Failed(e) = &s.state {
                    *e == error
                } else {
                    false
                }
            })
            .unwrap_or_default()
    })
}
