/// Creates aliases for the commonly used service types.
/// ## Parameters:
/// $t: The service's name. This is expected to be a plain type label passed in.
/// The macro will create the necessary structs for you.
///
/// $d: The data type. It must implement [ServiceData].
///
/// $e: The error type. It must implement [ServiceError].
///
/// ## Example usage
/// ```rust, ignore
/// use q_service::prelude::*;
/// use bevy::prelude::*;
///
/// #[derive(ServiceData, Clone, Default, PartialEq, Debug)]
/// struct MyData;
/// #[derive(ServiceError, thiserror::Error, PartialEq, Debug, Clone)]
/// enum MyError {}
///
/// service!(Example, MyData, MyError);
///
/// let app = App::new();
/// app.add_service(EXAMPLE_SERVICE_SPEC);
/// ```
#[macro_export]
macro_rules! service {
    ($t:ty, $d:ty, $e:ty) => {
        $crate::paste::paste! {
            mod [<$t:lower:snake _alias_impl>] {
                use super::*;
                use $crate::prelude::*;
                use std::marker::PhantomData;
                /// Label for the state. Works as part of a unique identifier.
                #[derive(ServiceLabel, PartialEq, Eq, Debug, Copy, Clone, Hash)]
                pub struct [<$t Label>];
                pub type [<$t Spec>]= ServiceSpec<[<$t Label>], $d, $e>;
                pub type [<$t>] = Service<[<$t Label>], $d, $e>;
                pub type [<$t Hooks>] = ServiceHooks<$e>;
                /// Track service state changes. Inner value is a tuple, (previous_state, current_state).
                pub type [<$t StateChange>] = ServiceStateChange<[<$t Label>], $d, $e>;
                pub type [<Enter $t State>] = EnterServiceState<[<$t Label>], $d, $e>;
                pub type [<Exit $t State>] = ExitServiceState<[<$t Label>], $d, $e>;
                pub type [<$t Enabled>] = ServiceEnabled<[<$t Label>], $d, $e>;
                pub type [<$t Disabled>] = ServiceDisabled<[<$t Label>], $d, $e>;
                pub type [<$t Initialized>] = ServiceInitialized<[<$t Label>], $d, $e>;
                pub type [<$t Failed>] = ServiceFailed<[<$t Label>], $d, $e>;
                pub const [<$t:snake:upper >]: ServiceHandle<[<$t Label>], $d, $e> = ServiceHandle::<[<$t Label>], $d, $e>::const_default();
                pub const [<$t:snake:upper _SPEC>]: ServiceSpec<[<$t Label>], $d, $e> = ServiceSpec::<[<$t Label>], $d, $e>::const_default();
            }
            pub use [< $t:lower _alias_impl >]::*;
        }
    };
}
