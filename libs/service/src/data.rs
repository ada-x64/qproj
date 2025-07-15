use crate::{graph::ServiceDepInfo, prelude::*};
use std::{
    any::{TypeId, type_name},
    error::Error,
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
};

/// A type which can be used as the unique identifer of a service.
/// Note that this _must be unique,_ otherwise instantiating a service with this
/// label will override any previous such services.
/// ```rust
/// #[derive(ServiceLabel, Clone, PartialEq, Eq, Debug, Hash)]
/// pub struct MyService;
/// ```
pub trait ServiceLabel:
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

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceErrorKind<E>
where
    E: ServiceError,
{
    #[error("{0}")]
    Own(E),
    #[error("Dependency {0} of {1} failed with error {2}")]
    Dependency(String, String, String),
}
impl<E: ServiceError> ServiceError for ServiceErrorKind<E> {}

/// A handle for the given service.
#[derive(Debug, Default, Clone, Copy)]
pub struct ServiceHandle<T, D, E>(
    pub PhantomData<T>,
    pub PhantomData<D>,
    pub PhantomData<E>,
)
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError;
impl<T, D, E> ServiceHandle<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    pub const fn const_default() -> Self {
        Self(PhantomData, PhantomData, PhantomData)
    }
    pub fn from_service(_: &Service<T, D, E>) -> Self {
        Self::const_default()
    }
    pub fn from_spec(_: &ServiceSpec<T, D, E>) -> Self {
        Self::const_default()
    }
}
impl<T, D, E> Display for ServiceHandle<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // of form "some::path::to::service_impl::MyServiceLabel"
        let mut base = type_name::<T>();
        let last_colon = base.rfind(':');
        if let Some(idx) = last_colon {
            base = base.split_at(idx + 1).1;
        }
        f.write_str(base.split_at(base.len() - 5).0)
    }
}

/// Automatically derived for service handles.
pub trait IsServiceHandle {}
impl<T, D, E> IsServiceHandle for ServiceHandle<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
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
    Failed(ServiceErrorKind<E>),
}
