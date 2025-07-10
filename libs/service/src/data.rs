use std::{fmt::Debug, hash::Hash};

use bevy::{platform::collections::HashMap, prelude::*};

use crate::lifecycle::*;

#[derive(Component, Debug, Default)]
pub struct ServiceManager;

/// Maps service dependencies. (A,B) means A depends on B.
// TODO: Cycle detection.
#[derive(Resource, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct ServiceDependencies<T: ServiceNames>(pub HashMap<T, Vec<T>>);
impl<T: ServiceNames> FromWorld for ServiceDependencies<T> {
    fn from_world(_world: &mut World) -> Self {
        Self(HashMap::default())
    }
}

/// Marker trait for services.
#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Deref, DerefMut)]
pub struct Service<T: ServiceNames>(T);

/// Ideally this should be an enum.
pub trait ServiceNames:
    Debug + PartialEq + Eq + Hash + Sync + Send + Clone + Copy + 'static
{
}
impl<T> ServiceNames for T where
    T: Debug + PartialEq + Eq + Hash + Sync + Send + Clone + Copy + 'static
{
}

/// Tracks the current state of the service.
#[derive(Default, Debug)]
pub enum ServiceStatus {
    #[default]
    Uinitialized,
    Initializing,
    Enabled,
    Disabled,
    Failed(BevyError),
}

/// Wrapper for [ServiceStatus].
#[derive(Default, Component, Debug, Deref, DerefMut)]
pub struct ServiceState(ServiceStatus);

/// Used to instantiate services.
#[derive(Bundle, Debug)]
pub struct ServiceBundle<T: ServiceNames> {
    service: Service<T>,
    state: ServiceState,
}
impl<T: ServiceNames> From<&ServiceSpec<T>> for ServiceBundle<T> {
    fn from(value: &ServiceSpec<T>) -> Self {
        Self {
            service: Service(value.name),
            state: ServiceState::default(),
        }
    }
}

/// Use this to specify a new service.
#[derive(Clone, Debug)]
pub struct ServiceSpec<T: ServiceNames> {
    /// The service to add.
    pub name: T,
    /// [TypeId]s of the service's dependencies.
    pub deps: Vec<T>,
    /// Does this service begin immediately?
    pub is_startup: bool,
    /// Lifecycle hooks
    pub lifecycle: ServiceLifecycle,
}
impl<T: ServiceNames> ServiceSpec<T> {
    /// Creates a new simple service with no dependencies.
    pub fn new(service: T) -> Self {
        Self {
            name: service,
            deps: vec![],
            is_startup: false,
            lifecycle: ServiceLifecycle::default(),
        }
    }
    pub fn with_startup(self, is_startup: bool) -> Self {
        Self { is_startup, ..self }
    }
    pub fn with_deps(self, deps: Vec<T>) -> Self {
        Self { deps, ..self }
    }
    pub fn on_init(self, on_init: InitFn) -> Self {
        Self {
            lifecycle: ServiceLifecycle {
                on_init,
                ..self.lifecycle
            },
            ..self
        }
    }
    pub fn on_enable(self, on_enable: EnableFn) -> Self {
        Self {
            lifecycle: ServiceLifecycle {
                on_enable,
                ..self.lifecycle
            },
            ..self
        }
    }
    pub fn on_disable(self, on_disable: EnableFn) -> Self {
        Self {
            lifecycle: ServiceLifecycle {
                on_disable,
                ..self.lifecycle
            },
            ..self
        }
    }
    pub fn on_failure(self, on_failure: FailFn) -> Self {
        Self {
            lifecycle: ServiceLifecycle {
                on_failure,
                ..self.lifecycle
            },
            ..self
        }
    }
}
