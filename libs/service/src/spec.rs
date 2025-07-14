use crate::prelude::*;
use bevy::prelude::*;

/// Use this to specify a new service.
#[derive(Debug, Default)]
pub struct ServiceSpec<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    /// The service's dependencies.
    pub deps: Vec<T>,
    /// Does this service begin immediately?
    /// Note that this will run as soon as the first commands are run.
    /// This may be before Startup!
    pub is_startup: bool,
    /// The data provided on startup.
    pub initial_data: Option<D>,
    /// Lifecycle hook
    pub on_init: Option<InitFn<E>>,
    /// Lifecycle hook
    pub on_enable: Option<EnableFn<E>>,
    /// Lifecycle hook
    pub on_disable: Option<DisableFn<E>>,
    /// Lifecycle hook
    pub on_failure: Option<FailureFn<E>>,
}
macro_rules! on {
    ($($name:ident),*) => {
        $crate::paste::paste! {
            $(
                pub fn [<on_ $name:snake:lower>]<M>(self, s: impl [<Into $name:camel Fn>]<E, M>) -> Self {
                    Self {
                        [< on_ $name:snake:lower >]: Some([<$name:camel Fn>]::new(s)),
                        ..self
                    }
                }
            )*
        }
    };
}
impl<T: ServiceLabel, D: ServiceData, E: ServiceError> ServiceSpec<T, D, E> {
    /// Creates a new simple service with no dependencies.
    pub const fn const_default() -> Self {
        Self {
            deps: vec![],
            is_startup: false,
            initial_data: None,
            on_init: None,
            on_enable: None,
            on_disable: None,
            on_failure: None,
        }
    }
    on!(Init, Enable, Disable, Failure);
    /// Initialize the function on Startup
    pub fn is_startup(self, is_startup: bool) -> Self {
        Self { is_startup, ..self }
    }
    pub fn with_deps(self, deps: Vec<T>) -> Self {
        Self { deps, ..self }
    }
    pub fn with_data(self, data: D) -> Self {
        Self {
            initial_data: Some(data),
            ..self
        }
    }
}
