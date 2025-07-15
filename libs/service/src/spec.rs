use crate::prelude::*;
use bevy::prelude::*;

/// Use this to specify a new service.
#[derive(Debug, Default)]
pub struct ServiceSpec<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    pub(crate) handle: ServiceHandle<T, D, E>,
    /// The service's type-erased dependencies.
    pub(crate) deps: Vec<Box<dyn IsServiceDep>>,
    /// Does this service begin immediately?
    /// Note that this will run as soon as the first commands are run.
    /// This may be before Startup!
    pub(crate) is_startup: bool,
    /// The data provided on startup.
    pub(crate) initial_data: Option<D>,
    /// Lifecycle hook
    pub(crate) on_init: Option<InitFn<E>>,
    /// Lifecycle hook
    pub(crate) on_enable: Option<EnableFn<E>>,
    /// Lifecycle hook
    pub(crate) on_disable: Option<DisableFn<E>>,
    /// Lifecycle hook
    pub(crate) on_failure: Option<FailureFn<E>>,
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
            handle: ServiceHandle::const_default(),
            deps: vec![],
            is_startup: false,
            initial_data: None,
            on_init: None,
            on_enable: None,
            on_disable: None,
            on_failure: None,
        }
    }

    // Hook setters.
    on!(Init, Enable, Disable, Failure);

    /// Initialize the function on Startup
    pub fn is_startup(self, is_startup: bool) -> Self {
        Self { is_startup, ..self }
    }
    /// Add dependencies.
    pub fn with_deps(self, deps: Vec<impl IsServiceDep + 'static>) -> Self {
        let deps = deps
            .into_iter()
            .map(|d| Box::new(d) as Box<dyn IsServiceDep>)
            .collect();
        Self { deps, ..self }
    }
    /// Insert data to be available on initialization.
    pub fn with_data(self, data: D) -> Self {
        Self {
            initial_data: Some(data),
            ..self
        }
    }
}
