use crate::{graph::DependencyGraph, prelude::*};
use bevy::prelude::*;

macro_rules! events {
    ($app:ident, $($name:ident $(,)?)* ) => {
        $(
            $app.add_event::<$name<T, D, E>>();
        )*
    }
}

macro_rules! observers {
    ($app:ident, $( ( $name:ident $(, $err:ty )* )$(,)?)*) => {
        $(
            $crate::paste::paste! {
                $app.add_observer(
                    |trigger: Trigger<$name<T, D, E>>,
                     mut commands: Commands| {
                         commands.[<$name:snake:lower>](trigger.event().0.clone(), $(trigger.event().1.clone() as $err)*);
                    },
                );
            }
        )*
    };
}

pub trait ServiceExt<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self;
}
impl<T: ServiceLabel, D: ServiceData, E: ServiceError> ServiceExt<T, D, E>
    for App
{
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self {
        debug!("Adding service {}", std::any::type_name::<T>());

        use crate::lifecycle::events::{
            DisableService, EnableService, FailService, InitService,
        };

        events!(
            self,
            EnterServiceState,
            ExitServiceState,
            EnableService,
            DisableService,
            InitService,
            FailService,
        );
        observers!(
            self,
            (EnableService),
            (DisableService),
            (InitService),
            (FailService, E),
        );

        let world = self.world_mut();

        if world.get_resource::<DependencyGraph>().is_none() {
            world.init_resource::<DependencyGraph>();
        }
        if world.get_resource::<Service<T, D, E>>().is_some() {
            warn!(
                "Tried to add already existing service {:?}",
                std::any::type_name::<T>()
            );
            return self;
        }
        let is_startup = spec.is_startup;
        if let Err(e) = world
            .resource_mut::<DependencyGraph>()
            .add_service_from_spec(
                ServiceHandle::<T, D, E>::const_default(),
                spec.deps.clone(),
            )
        {
            panic!("{e}");
        }
        let service = Service::from_spec(spec);
        world.insert_resource(service);
        if is_startup {
            world.schedule_scope(Startup, |_world, sched| {
                debug!("initializing {} at startup", std::any::type_name::<T>());
                sched.add_systems(move |mut commands: Commands| {
                    commands.init_service(ServiceHandle::<T, D, E>::const_default());
                });
            });
        }
        self
    }
}
