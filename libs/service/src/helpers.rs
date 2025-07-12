use crate::prelude::*;
use bevy::prelude::*;

/// Creates aliases for the commonly used service types.
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
macro_rules! alias {
    ($prefix:ident, $e:ty) => {
        alias!($prefix, String, (), $e);
    };
    ($prefix:ident, $t:ty, $e:ty) => {
        alias!($prefix, $t, (), $e);
    };
    ($prefix:ident, $t:ty, $d:ty, $e:ty) => {
        $crate::paste::paste! {
            mod [<$prefix:lower:snake _alias_impl>] {
                use super::*;
                use $crate::prelude::*;
                use std::marker::PhantomData;
                /// Spec for the service. See [ServiceSpec].
                pub type [<$prefix ServiceSpec>]= ServiceSpec<$t, $d, $e>;
                /// The main service component for the service. See [Service].
                pub type [<$prefix Service>] = Service<$t, $d, $e>;
                /// Lifecycle hooks for the service. See [ServiceHooks].
                pub type [<$prefix ServiceHooks>] = ServiceHooks<$e>;
                /// A marker which uniquely points out the service. See [ServiceMarker].
                pub const [<$prefix:snake:upper _SERVICE_MARKER>]: ServiceMarker<$t, $d, $e> = ServiceMarker::<$t, $d, $e>(PhantomData, PhantomData, PhantomData);
            }
            pub use [< $prefix:lower _alias_impl >]::*;
        }
    };
}

/// Run condition which checks if the given service has the given state.
pub fn service_has_state<T, D, E>(
    name: T,
    state: ServiceState<E>,
    _marker: ServiceMarker<T, D, E>,
) -> impl Condition<()>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    IntoSystem::into_system(move |services: Query<&Service<T, D, E>>| {
        services
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.state.eq(&state))
            .unwrap_or_default()
    })
}
