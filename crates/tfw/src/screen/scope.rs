use std::marker::PhantomData;

#[allow(unused_imports, reason = "used in docs")]
use bevy::app::FixedMain;
use bevy::{ecs::system::ScheduleSystem, platform::collections::HashMap};

pub use crate::prelude::*;

mod data {
    use std::marker::PhantomData;

    use bevy::ecs::schedule::ScheduleLabel;

    use crate::prelude::*;

    /// Specifies the order of execution for a schedule.
    #[derive(Default, Debug)]
    pub enum Order {
        #[default]
        Before,
        After,
    }

    /// Manually triggered schedule which is called when the screen is unloaded.
    #[derive(ScheduleLabel, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
    pub struct UnloadSchedule;

    /// Interal. Called during [UnloadSchedule]
    #[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
    pub struct UnloadSystems<S: Screen>(PhantomData<S>);

    /// Interal. Called after [UnloadSchedule]
    #[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
    pub struct PostUnloadSystems<S: Screen>(PhantomData<S>);
}
pub use data::Order;
pub(crate) use data::UnloadSchedule;
use data::*;

/// On build, this will initialize a new [Schedule]. The newly created schedule
/// has a [SystemSet] associated with it which is scoped to run only if the
/// world is in the given ReadyState. Schedules can run in either [Main] or
/// [FixedMain]. The given systems will run after [Update] or
/// [FixedUpdate].
pub struct ScreenScopeBuilder<'a, S>
where
    S: Screen,
{
    schedules: HashMap<ScreenScheduleKind, Schedule>,
    app: &'a mut App,
    order: Order,
    fixed_order: Order,
    _ghost: PhantomData<S>,
}

impl<'a, S> ScreenScopeBuilder<'a, S>
where
    S: Screen,
{
    pub fn new(app: &'a mut App) -> Self {
        Self {
            schedules: HashMap::default(),
            order: Order::default(),
            fixed_order: Order::default(),
            app,
            _ghost: PhantomData,
        }
    }

    /// Sets when this screen's systems run relative to the [Update] in the [Main] schedule
    pub fn with_order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }

    /// Sets when this screen's fixed systems run relative to the [FixedMain]
    /// schedule
    pub fn with_fixed_order(mut self, order: Order) -> Self {
        self.fixed_order = order;
        self
    }

    /// Add systems to the schedule scope. Will run before or after [Update]
    /// according to the builder's [Order]
    pub fn add_systems<M>(
        mut self,
        kind: ScreenScheduleKind,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        self.schedules
            .entry(kind)
            .or_insert(Schedule::new(ScreenScheduleLabel::<S>::new(kind)))
            .add_systems(systems);
        self
    }

    /// Adds a system which will run when the screen finishes loading its
    /// systems. This is a shorthand for
    /// `app.add_systems(OnScreenReady::<S>::default(), systems)`.
    pub fn on_ready<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(OnScreenReady::<S>::default(), systems);
        self
    }

    pub fn on_unload<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.init_schedule(UnloadSchedule);
        self.app.add_systems(
            UnloadSchedule,
            systems.in_set(UnloadSystems::<S>::default()),
        );
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
                state: ScreenStateKind::Unloaded,
            },
        );

        // watch screen switcher
        app.add_observer(on_switch_screen::<S>);

        // scope systems
        let blocking = S::has_assets() && S::STRATEGY == LoadingStrategy::Blocking;
        for schedule in self.schedules.into_iter() {
            let label = schedule.1.label();
            let kind = schedule.0;
            app.add_schedule(schedule.1);
            let system = move |mut commands: Commands, state: ScreenState<S>| {
                let bypass = !blocking
                    && matches!(kind, ScreenScheduleKind::Fixed | ScreenScheduleKind::Main)
                    && *state != ScreenStateKind::Unloaded;
                if bypass || *state == kind.into() {
                    commands.run_schedule(label);
                }
            };
            if matches!(kind, ScreenScheduleKind::Fixed) {
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

        // set up unload schedule
        app.configure_sets(
            UnloadSchedule,
            (
                PostUnloadSystems::<S>::default().after(UnloadSystems::<S>::default()),
                UnloadSystems::<S>::default()
                    .run_if(not(screen_has_state::<S>(ScreenStateKind::Unloaded))),
            ),
        );

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

fn unload<S: Screen>(mut screen_state: ScreenStateMut<S>) {
    debug!("UnloadSystems {:?}", S::name());
    screen_state.unload();
}

/// This function clears out all the non-screen-scoped entities.
fn post_unload<S: Screen>(
    mut commands: Commands,
    mut next_state: ScreenStateMut<S>,
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
