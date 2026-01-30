use std::marker::PhantomData;

pub use crate::prelude::*;
use bevy::{ecs::system::ScheduleSystem, platform::collections::HashMap};

// TODO: DOCUMENT ME
pub struct ScreenScopeBuilder<'a, S>
where
    S: Screen,
{
    schedules: HashMap<ScreenSchedule, Schedule>,
    app: &'a mut App,
    _ghost: PhantomData<S>,
}

impl<'a, S> ScreenScopeBuilder<'a, S>
where
    S: Screen,
{
    pub fn new(app: &'a mut App) -> Self {
        Self {
            schedules: HashMap::default(),
            app,
            _ghost: PhantomData,
        }
    }

    /// Add systems to the schedule scope. Will run before or after [Update]
    /// according to the builder's [Order]
    pub fn add_systems<M>(
        mut self,
        kind: ScreenSchedule,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        self.schedules
            .entry(kind)
            .or_insert(Schedule::new(ScreenScheduleLabel::<S>::new(kind)))
            .add_systems(systems);
        self
    }

    /// Runs once on screen load. Shorthand for
    /// `app.add_systems(OnScreenLoad::<S>::default(), systems)`.
    pub fn on_load<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(OnScreenLoad::<S>::default(), systems);
        self
    }
    /// Runs once on screen ready. Shorthand for
    /// `app.add_systems(OnScreenReady::<S>::default(), systems)`.
    pub fn on_ready<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(OnScreenReady::<S>::default(), systems);
        self
    }
    /// Runs once when screen begins to unload. Shorthand for
    /// `app.add_systems(OnScreenUnload::<S>::default(), systems)`.
    pub fn on_unload<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app
            .add_systems(OnScreenUnload::<S>::default(), systems);
        self
    }
    /// Runs once when screen finished unloading. Shorthand for
    /// `app.add_systems(OnScreenUnloaded::<S>::default(), systems)`.
    pub fn on_unloaded<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app
            .add_systems(OnScreenUnloaded::<S>::default(), systems);
        self
    }

    /// Builds the schedule and adds it to the app.
    pub fn build(self) {
        let app = self.app;
        debug!("Building screen {:?}", S::name());

        // insert data
        app.init_resource::<S::SETTINGS>();
        let id = app.world_mut().register_component::<S>();
        let mut registry = app.world_mut().get_resource_or_init::<ScreenRegistry>();
        registry.insert(
            id,
            ScreenData {
                name: S::name(),
                id,
                state: ScreenState::Unloaded,
            },
        );

        // watch screen switcher
        app.add_observer(on_switch_screen::<S>);

        // scope systems
        let blocking = S::has_assets() && S::STRATEGY == LoadingStrategy::Blocking;
        for (screen_schedule, schedule) in self.schedules.into_iter() {
            let label = schedule.label();
            app.add_schedule(schedule);
            let system = move |mut commands: Commands, data: ScreenDataRef<S>| {
                let bypass = !blocking
                    && matches!(
                        screen_schedule,
                        ScreenSchedule::Fixed | ScreenSchedule::Main
                    )
                    && data.state != ScreenState::Unloaded;
                if bypass || data.state == screen_schedule.into() {
                    commands.run_schedule(label);
                }
            };
            if matches!(screen_schedule, ScreenSchedule::Fixed) {
                app.add_systems(FixedPreUpdate, system);
            } else {
                app.add_systems(PreUpdate, system);
            };
        }

        // NOTE: Because this is no longer a state-based sysyt
        // if S::has_assets() {
        //     debug!("Adding loading state for {}", S::name());
        //     app.add_loading_state(
        //         LoadingState::new(ScreenState::<S>::loading())
        //             .continue_to_state(ScreenState::<S>::ready())
        //             .load_collection::<S::ASSETS>(),
        //     );
        // }

        // // set up unload schedule
        // app.configure_sets(
        //     UnloadSchedule,
        //     (
        //         PostUnloadSystems::<S>::default().after(UnloadSystems::<S>::default()),
        //         UnloadSystems::<S>::default()
        //             .run_if(not(screen_has_state::<S>(ScreenState::Unloaded))),
        //     ),
        // );

        // spawn on load
        app.add_systems(OnScreenLoad::<S>::default(), S::spawn);

        // Lifecycle
        #[cfg(debug_assertions)]
        {
            app.add_systems(OnScreenLoad::<S>::default(), || {
                debug!("Loading {:?}", S::name())
            });
            app.add_systems(OnScreenReady::<S>::default(), || {
                debug!("Ready {:?}", S::name())
            });
            app.add_systems(OnScreenUnload::<S>::default(), || {
                debug!("Unloading {:?}", S::name())
            });
            app.add_systems(OnScreenUnloaded::<S>::default(), || {
                debug!("Unloaded {:?}", S::name())
            });
        }
    }
}

fn unload<S: Screen>(mut screen_state: ScreenDataMut<S>) {
    debug!("UnloadSystems {:?}", S::name());
    screen_state.unload();
}

/// This function clears out all the non-screen-scoped entities.
fn post_unload<S: Screen>(
    mut commands: Commands,
    mut next_state: ScreenDataMut<S>,
    // Any entity which is (explicitly marked as ScreenScoped, or is _not_ marked
    // as persistent) _and_ is not a top-level observer
    screen_scoped: Query<
        Entity,
        (
            Or<(
                With<ScreenScoped>,  // is explicitly screen-scoped
                Without<Persistent>, // is explicitly persistent
            )>,
            Not<(Or<(With<Observer>, With<Window>)>, Without<ChildOf>)>, // top-level items
        ),
    >,
) {
    debug!("PostUnloadSystems {:?}", S::name());
    screen_scoped.iter().for_each(|e| {
        commands.entity(e).despawn();
    });
    next_state.unload();
}
