use std::{error::Error, fmt::Debug, hash::Hash, marker::PhantomData};

use bevy::{platform::collections::HashMap, prelude::*};

use crate::lifecycle::*;

#[derive(Component, Debug, Default)]
pub struct ServiceManager;

/// Maps service dependencies. (A,B) means A depends on B.
// TODO: Cycle detection.
#[derive(Resource, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct ServiceDependencies<T: ServiceName>(pub HashMap<T, Vec<T>>);
impl<T: ServiceName> FromWorld for ServiceDependencies<T> {
    fn from_world(_world: &mut World) -> Self {
        Self(HashMap::default())
    }
}

/// A component which represents a service.
/// These are typically singleton entities.
// TODO: Better docs. This is the primary IDE interface for information about
// the crate.
#[derive(Component, Debug)]
pub struct Service<T: ServiceName, D: ServiceData, E: ServiceError> {
    pub name: T,
    pub data: D,
    pub hooks: ServiceHooks<E>,
    pub state: ServiceState<E>,
}
impl<T: ServiceName, D: ServiceData, E: ServiceError> Service<T, D, E> {
    pub fn from_spec(spec: ServiceSpec<T, D, E>) -> Self {
        Self {
            name: spec.name,
            data: spec.initial_data,
            hooks: spec.hooks,
            state: ServiceState::default(),
        }
    }
}

pub(crate) struct PhantomService<T, D, E>(
    PhantomData<T>,
    PhantomData<D>,
    PhantomData<E>,
)
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError;
impl<T, D, E> Default for PhantomService<T, D, E>
where
    T: ServiceName,
    D: ServiceData,
    E: ServiceError,
{
    fn default() -> Self {
        Self(PhantomData, PhantomData, PhantomData)
    }
}

// /// A unique identifier / marker struct for a service.
// pub trait ServiceName:
//     Debug + PartialEq + Eq + Hash + Sync + Send + Clone + Copy + 'static
// {
// }
// impl<T> ServiceName for T where
//     T: Debug + PartialEq + Eq + Hash + Sync + Send + Clone + Copy + 'static
// {
// }

pub trait ServiceName:
    Send + Sync + Clone + PartialEq + Eq + Debug + Hash + 'static
{
}
impl<T> ServiceName for T where
    T: Send + Sync + Clone + PartialEq + Eq + Debug + Hash + 'static
{
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

pub trait ServiceData:
    Clone + Debug + PartialEq + Eq + Hash + Send + Sync + Default + 'static
{
}
impl<T> ServiceData for T where
    T: Clone + Debug + Send + Sync + Default + PartialEq + Eq + Hash + 'static
{
}

pub trait ServiceError: Error + Clone + Send + Sync + 'static {}
impl<T> ServiceError for T where T: Error + Clone + Send + Sync + 'static {}

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
    pub initial_data: D,
}
impl<T: ServiceName, D: ServiceData, E: ServiceError> ServiceSpec<T, D, E> {
    /// Creates a new simple service with no dependencies.
    pub fn new(service: T) -> Self {
        Self {
            name: service,
            deps: vec![],
            is_startup: false,
            hooks: ServiceHooks::default(),
            initial_data: D::default(),
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
            initial_data: data,
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
