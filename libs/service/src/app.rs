use crate::prelude::*;
use bevy::prelude::*;

pub trait ServiceExt<T: ServiceLabel, D: ServiceData, E: ServiceError> {
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self;
}
impl<T: ServiceLabel, D: ServiceData, E: ServiceError> ServiceExt<T, D, E>
    for App
{
    fn add_service(&mut self, spec: ServiceSpec<T, D, E>) -> &mut Self {
        debug!("Adding service {}", std::any::type_name::<T>());

        events::<T, D, E>(self);

        let world = self.world_mut();
        if world.get_resource::<Service<T, D, E>>().is_some() {
            warn!(
                "Tried to add already existing service {:?}",
                std::any::type_name::<T>()
            );
            return self;
        }
        let is_startup = spec.is_startup;
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

fn events<T, D, E>(app: &mut App) -> &mut App
where
    T: ServiceLabel,
    D: ServiceData,
    E: ServiceError,
{
    use crate::lifecycle::events::{
        DisableService, EnableService, FailService, InitService,
    };
    app.add_event::<ServiceStateChange<T, E>>()
        .add_event::<EnterServiceState<T, E>>()
        .add_event::<ExitServiceState<T, E>>()
        .add_event::<EnableService<T, D, E>>()
        .add_event::<DisableService<T, D, E>>()
        .add_event::<InitService<T, D, E>>()
        .add_event::<FailService<T, D, E>>()
        .add_observer(
            |trigger: Trigger<EnableService<T, D, E>>,
             mut commands: Commands| {
                commands.enable_service(trigger.event().0.clone());
            },
        )
        .add_observer(
            |trigger: Trigger<DisableService<T, D, E>>,
             mut commands: Commands| {
                commands.disable_service(trigger.event().0.clone());
            },
        )
        .add_observer(
            |trigger: Trigger<InitService<T, D, E>>, mut commands: Commands| {
                commands.init_service(trigger.event().0.clone());
            },
        )
        .add_observer(
            |trigger: Trigger<FailService<T, D, E>>, mut commands: Commands| {
                commands.fail_service(
                    trigger.event().0.clone(),
                    trigger.event().1.clone(),
                );
            },
        )
}
