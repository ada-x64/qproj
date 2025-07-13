use std::{error::Error, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy::prelude::*;

use crate::lifecycle::*;

/// A type which can be used as the unique identifer of a service.
/// This should be an enum, although we can't enforce that at the trait level.
/// However, if you use the `service!` macro in this crate, it will be assumed.
pub trait ServiceName:
    Send + Sync + Clone + PartialEq + Eq + Debug + Hash + 'static
{
}

/// An arbitrary data type which can be used as extra state information for a
/// service.
pub trait ServiceData:
    Clone + Debug + PartialEq + Default + Send + Sync + 'static
{
}
impl ServiceData for () {}

/// The error type for a service.
pub trait ServiceError:
    Error + Clone + PartialEq + Send + Sync + 'static
{
}

/// A component which represents a service.
/// These are typically singleton entities.
// TODO: Better docs. This is the primary IDE interface for information about
// the crate.
#[derive(Component, Debug)]
pub struct Service<T: ServiceName, D: ServiceData, E: ServiceError> {
    /// The unique name of a service. This will be used for all access checks.
    pub name: T,
    /// Arbitrary data store.
    pub data: D,
    /// Lifecycle hooks.
    pub hooks: ServiceHooks<E>,
    /// The current state of the service.
    pub state: ServiceState<E>,
}
impl<T: ServiceName, D: ServiceData, E: ServiceError> Service<T, D, E> {
    pub fn from_spec(spec: ServiceSpec<T, D, E>) -> Self {
        Self {
            name: spec.name,
            data: spec.initial_data.unwrap_or_default(),
            hooks: spec.hooks,
            state: ServiceState::default(),
        }
    }
}

/// A handle for the given service.
pub struct ServiceHandle<T, D, E>(
    pub T,
    pub PhantomData<D>,
    pub PhantomData<E>,
)
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError;
impl<T, D, E> ServiceHandle<T, D, E>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    pub const fn new(name: T) -> Self {
        Self(name, PhantomData, PhantomData)
    }
    pub const fn name(&self) -> &T {
        &self.0
    }
}

/// Tracks the current state of the service.
/// This does not use the built-in States trait.
/// In order to hook into changes, use events or service hooks.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum ServiceState<E: ServiceError> {
    #[default]
    Uninitialized,
    Initializing,
    Enabled,
    Disabled,
    Failed(E),
}

/// Use this to specify a new service.
#[derive(Clone, Debug)]
pub struct ServiceSpec<T: ServiceName, D: ServiceData, E: ServiceError> {
    /// The service to add.
    pub name: T,
    /// The service's dependencies.
    pub deps: Vec<T>,
    /// Does this service begin immediately?
    pub is_startup: bool,
    /// Lifecycle hooks
    pub hooks: ServiceHooks<E>,
    pub initial_data: Option<D>,
}
impl<T: ServiceName, D: ServiceData, E: ServiceError> ServiceSpec<T, D, E> {
    /// Creates a new simple service with no dependencies.
    pub const fn new(name: T) -> Self {
        Self {
            name,
            deps: vec![],
            is_startup: false,
            hooks: ServiceHooks::const_default(),
            initial_data: None, // this allows the fn to be const
        }
    }
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
    pub fn on_init(self, on_init: InitFn<E>) -> Self {
        Self {
            hooks: self.hooks.on_init(on_init),
            ..self
        }
    }
    pub fn on_enable(self, on_enable: EnableFn<E>) -> Self {
        Self {
            hooks: self.hooks.on_enable(on_enable),
            ..self
        }
    }
    pub fn on_disable(self, on_disable: EnableFn<E>) -> Self {
        Self {
            hooks: self.hooks.on_disable(on_disable),
            ..self
        }
    }
    pub fn on_failure(self, on_failure: FailFn<E>) -> Self {
        Self {
            hooks: self.hooks.on_failure(on_failure),
            ..self
        }
    }
}
