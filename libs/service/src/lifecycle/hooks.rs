use crate::data::*;
use bevy::{ecs::system::*, prelude::*};

macro_rules! hooks {
    ($(($name: ident, $in:ty, $out:ty, $default:expr ),)*) => {
        $crate::paste::paste! {
            $(
                #[derive(Deref, DerefMut, Debug)]
                pub struct [<$name Fn>]<E: ServiceError>(
                    Box<dyn System<In = $in, Out = $out>>,
                );
                impl<E: ServiceError> [<$name Fn>]<E> {
                    pub fn new<M, S: IntoSystem<$in, $out, M>>(s: S) -> Self {
                        Self(Box::new(IntoSystem::into_system(s)))
                    }
                }
                impl<E: ServiceError> Default for [<$name Fn>]<E> {
                    fn default() -> Self {
                        Self::new($default)
                    }
                }
                pub trait [<Into $name Fn>]<E: ServiceError, M>:
                    IntoSystem<$in, $out, M>
                {
                }
                impl<E: ServiceError, M, T> [<Into $name Fn>]<E, M> for T where
                    T: IntoSystem<$in, $out, M>
                {
                }
            )*
        }
    };
}

hooks!(
    (Init, (), Result<bool, E>, || Ok(true)),
    (Enable, (), Result<(), E>, || Ok(())),
    (Disable, (), Result<(), E>, || Ok(())),
    (Failure, In<ServiceErrorKind<E>>, (), |e: In<ServiceErrorKind<E>>| {error!("Service error: {e:?}");}),
);

#[derive(Debug)]
pub struct ServiceHooks<E>
where
    E: ServiceError,
{
    pub on_init: InitFn<E>,
    pub on_enable: EnableFn<E>,
    pub on_disable: DisableFn<E>,
    pub on_failure: FailureFn<E>,
}
macro_rules! on {
    ($($name:ident),*) => {
        $crate::paste::paste! {
            $(
                pub fn [<on_ $name:snake:lower>]<S, M>(self, s: S) -> Self
                where
                    S: [<Into $name:camel Fn>]<E, M>
                {
                    Self {
                        [<on_ $name:snake:lower>]: [<$name Fn>]::new(s),
                        ..self
                    }
                }
            )*
        }
    };
}

impl<E: ServiceError> ServiceHooks<E> {
    on!(Init, Enable, Disable, Failure);
}
// note: E is not Default so can't derive this
impl<E: ServiceError> Default for ServiceHooks<E> {
    fn default() -> Self {
        Self {
            on_init: InitFn::default(),
            on_enable: EnableFn::default(),
            on_disable: DisableFn::default(),
            on_failure: FailureFn::default(),
        }
    }
}
