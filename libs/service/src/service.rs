use std::any::{TypeId, type_name_of_val};

use crate::{
    graph::{DependencyGraph, ServiceDepInfo},
    prelude::*,
};
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
    _handle: ServiceHandle<T, D, E>,
    deps: Vec<Box<dyn IsServiceDep>>,
    initialized: bool,
}

impl<T, D, E> Service<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
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
            _handle: ServiceHandle::const_default(),
            deps: spec.deps,
            initialized: false,
        }
    }

    /// Initializes the service. Depending on the result of the hook, it will
    /// then either enable or disable the service. Handles errors.
    pub fn on_init(
        &mut self,
        world: &mut World,
    ) -> Result<(), ServiceErrorKind<E>> {
        // TODO: on_init should allow asyncronous behavior.
        self.set_state(world, ServiceState::Initializing);

        // initialize dependencies
        for dep in self.deps.iter_mut() {
            let info = dep.info(world);
            if info.is_service && !info.is_initialized {
                if let Err(e) = dep.initialize(world) {
                    let error = ServiceErrorKind::Dependency(
                        ServiceHandle::from_service(self).to_string(),
                        info.display_name,
                        e.to_string(),
                    );
                    self.on_failure(world, error.clone());
                    return Err(error);
                }
            }
        }

        // run registered hook
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
                let error = ServiceErrorKind::Own(error);
                self.on_failure(world, error.clone());
                self.hooks.on_init.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Enables the service and handles errors.
    pub fn on_enable(
        &mut self,
        world: &mut World,
    ) -> Result<(), ServiceErrorKind<E>> {
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
                let error = ServiceErrorKind::Own(error);
                self.on_failure(world, error.clone());
                self.hooks.on_enable.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Disables the service and handles errors.
    pub fn on_disable(
        &mut self,
        world: &mut World,
    ) -> Result<(), ServiceErrorKind<E>> {
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
                let error = ServiceErrorKind::Own(error);
                self.on_failure(world, error.clone());
                self.hooks.on_disable.apply_deferred(world);
                Err(error)
            }
        }
    }
    /// Handles errors.
    pub fn on_failure(
        &mut self,
        world: &mut World,
        error: ServiceErrorKind<E>,
    ) {
        self.hooks.on_failure.initialize(world);
        self.hooks
            .on_failure
            .run_without_applying_deferred(error.clone(), world);
        self.set_state(world, ServiceState::Failed(error));
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

#[derive(thiserror::Error, Debug)]
pub enum DepInitErr {
    #[error("Service {0} failed to initialize with error {1}")]
    Service(String, String),
}

pub trait IsServiceDep: std::fmt::Debug + Send + Sync {
    /// Although this takes an exclusive borrow, please do not mutate world
    /// here. This is necessary for resource scoping.
    fn info(&self, world: &mut World) -> ServiceDepInfo;
    /// Initialize the dependency and update the dependency graph.
    fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr>;
}
impl<T, D, E> IsServiceDep for Service<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn info(&self, _world: &mut World) -> ServiceDepInfo {
        ServiceDepInfo {
            type_id: TypeId::of::<Self>(),
            display_name: ServiceHandle::from_service(self).to_string(),
            is_initialized: self.initialized,
            is_service: true,
        }
    }
    fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr> {
        let info = self.info(world);
        debug!("Initializing dependency: {info:#?}");
        let res = self.on_init(world).map_err(|e| {
            DepInitErr::Service(info.display_name.clone(), e.to_string())
        });

        // Update graph
        if world.get_resource::<DependencyGraph>().is_none() {
            world.init_resource::<DependencyGraph>();
        }
        world.resource_scope(|world, mut graph: Mut<DependencyGraph>| {
            graph.register_service(
                ServiceHandle::<T, D, E>::const_default(),
                self.deps.iter().map(|d| d.info(world)).collect(),
            ).expect("Failed to add dependencies in service spec!\n.. Spec = {spec:#?}")
        });

        world
            .resource_mut::<DependencyGraph>()
            .services
            .entry(info.type_id)
            .insert(info);
        res
    }
}
impl<T, D, E> IsServiceDep for ServiceHandle<T, D, E>
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    fn info(&self, world: &mut World) -> ServiceDepInfo {
        world.resource_scope(|world, service: Mut<Service<T, D, E>>| {
            service.info(world)
        })
    }
    fn initialize(&mut self, world: &mut World) -> Result<(), DepInitErr> {
        world.resource_scope(|world, mut service: Mut<Service<T, D, E>>| {
            service.initialize(world)
        })
    }
}
