use crate::prelude::*;
use bevy::prelude::*;

#[derive(Debug, Resource)]
/// Represents a service.
pub struct Service<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    /// Arbitrary data store.
    pub data: D,
    /// Lifecycle hooks.
    pub hooks: ServiceHooks<E>,
    /// The current state of the service.
    pub state: ServiceState<E>,
    handle: ServiceHandle<T, D, E>,
}
impl<T: ServiceLabel, D: ServiceData, E: ServiceError> Service<T, D, E> {
    pub fn from_spec(spec: ServiceSpec<T, D, E>) -> Self {
        Self {
            data: spec.initial_data.unwrap_or_default(),
            state: ServiceState::default(),
            hooks: ServiceHooks {
                on_init: spec.on_init.unwrap_or_default(),
                on_enable: spec.on_enable.unwrap_or_default(),
                on_disable: spec.on_disable.unwrap_or_default(),
                on_failure: spec.on_failure.unwrap_or_default(),
            },
            handle: ServiceHandle::const_default(),
        }
    }

    /// Initializes the service. Depending on the result of the hook, it will
    /// then either enable or disable the service. Handles errors.
    pub fn on_init(&mut self, world: &mut World) -> Result<(), E> {
        // TODO: on_init should allow asyncronous behavior.
        self.set_state(world, ServiceState::Initializing);
        self.hooks.on_init.initialize(world); // TODO: Does this clear state?
        let res = self.hooks.on_init.run_without_applying_deferred((), world);
        match res {
            Ok(val) => {
                if val {
                    let res = self.on_enable(world);
                    self.hooks.on_init.apply_deferred(world);
                    res
                } else {
                    let res = self.on_disable(world);
                    self.hooks.on_init.apply_deferred(world);
                    res
                }
            }
            Err(error) => {
                self.on_failure(world, error.clone());
                self.hooks.on_init.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Enables the service and handles errors.
    pub fn on_enable(&mut self, world: &mut World) -> Result<(), E> {
        self.hooks.on_enable.initialize(world);
        let res = self
            .hooks
            .on_enable
            .run_without_applying_deferred((), world);
        match res {
            Ok(val) => {
                self.set_state(world, ServiceState::Enabled);
                self.hooks.on_enable.apply_deferred(world);
                Ok(val)
            }
            Err(error) => {
                self.on_failure(world, error.clone());
                self.hooks.on_enable.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Disables the service and handles errors.
    pub fn on_disable(&mut self, world: &mut World) -> Result<(), E> {
        self.hooks.on_disable.initialize(world);
        let res = self
            .hooks
            .on_disable
            .run_without_applying_deferred((), world);
        match res {
            Ok(val) => {
                self.set_state(world, ServiceState::Disabled);
                self.hooks.on_disable.apply_deferred(world);
                Ok(val)
            }
            Err(error) => {
                self.on_failure(world, error.clone());
                self.hooks.on_disable.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Handles errors.
    pub fn on_failure(&mut self, world: &mut World, error: E) {
        self.hooks.on_failure.initialize(world);
        self.hooks
            .on_failure
            .run_without_applying_deferred(error.clone(), world);
        self.set_state(
            world,
            ServiceState::Failed(ServiceErrorKind::Own(error)),
        );
        self.hooks.on_failure.apply_deferred(world);
    }

    pub fn set_state(&mut self, world: &mut World, state: ServiceState<E>) {
        let old_state = self.state.clone();
        self.state = state.clone();
        world.trigger(ServiceStateChange::<T, D, E>::new((
            old_state.clone(),
            state.clone(),
        )));
        world.trigger(EnterServiceState::<T, D, E>::new(state));
        world.trigger(ExitServiceState::<T, D, E>::new(old_state));
    }
}
