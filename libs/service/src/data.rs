use std::{
    any::TypeId, error::Error, fmt::Debug, hash::Hash, marker::PhantomData,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ServiceErrorKind<E> {
    Own(E),
    Dependency(TypeId),
}

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
