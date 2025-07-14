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
        self.add_event::<ServiceStateChange<T, E>>();
        self.add_event::<EnterServiceState<T, E>>();
        self.add_event::<ExitServiceState<T, E>>();
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
