pub use crate::prelude::*;
use bevy::{ecs::system::ScheduleSystem, platform::collections::HashMap};
use strum::IntoEnumIterator;

// TODO: DOCUMENT ME
pub struct ScreenScopeBuilder<'a, S>
where
    S: Screen,
{
    schedules: HashMap<ScreenSchedule, Schedule>,
    app: &'a mut App,
    skip_load: Option<bool>,
    skip_unload: Option<bool>,
    load_strategy: LoadStrategy,
    _ghost: PhantomData<S>,
}

impl<'a, S> ScreenScopeBuilder<'a, S>
where
    S: Screen,
{
    pub fn new(app: &'a mut App) -> Self {
        let schedules = ScreenSchedule::iter()
            .map(|kind| (kind, Schedule::new(ScreenScheduleLabel::new::<S>(kind))))
            .collect::<HashMap<_, _>>();
        Self {
            schedules,
            app,
            skip_load: None,
            skip_unload: None,
            load_strategy: LoadStrategy::default(),
            _ghost: PhantomData,
        }
    }

    /// Initialize directly into ready state. By default this is true, unless
    /// there are systems present in the Load schedule.
    pub fn with_skip_load(mut self, val: bool) -> Self {
        self.skip_load = Some(val);
        self
    }
    /// Deinitialize directly into unloaded state. By default this is true,
    /// unless there are systems present in the Unload schedule.
    pub fn with_skip_unload(mut self, val: bool) -> Self {
        self.skip_unload = Some(val);
        self
    }
    /// Sets the [LoadingStrategy]. By default, this is Blocking.
    pub fn with_load_strategy(mut self, val: LoadStrategy) -> Self {
        self.load_strategy = val;
        self
    }

    /// Add systems to the schedule scope. Will run in the specified schedule.
    pub fn add_systems<M>(
        mut self,
        kind: ScreenSchedule,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> Self {
        self.schedules
            .entry(kind)
            .or_insert(Schedule::new(ScreenScheduleLabel::new::<S>(kind)))
            .add_systems(systems);
        self
    }

    /// Runs once on screen load. Shorthand for
    /// `app.add_systems(on_screen_load::<S>(), systems)`.
    /// Note that this _will not_ automatically enable loading for this screen.
    /// Make sure you either have systems in the [ScreenSchedule::Load](ScreenSchedule) schedule,
    /// or you manually set [with_skip_load(false)](ScreenScopeBuilder::with_skip_load)
    pub fn on_load<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(on_screen_load::<S>(), systems);
        self
    }
    /// Runs once on screen ready. Shorthand for
    /// `app.add_systems(on_screen_ready::<S>(), systems)`.
    pub fn on_ready<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(on_screen_ready::<S>(), systems);
        self
    }
    /// Runs once when screen begins to unload. Shorthand for
    /// `app.add_systems(on_screen_unload::<S>(), systems)`.
    /// Note that this _will not_ automatically enable unloading for this screen.
    /// Make sure you either have systems in the [ScreenSchedule::Unload](ScreenSchedule) schedule,
    /// or you manually set [with_skip_unload(false)](ScreenScopeBuilder::with_skip_unload)
    pub fn on_unload<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(on_screen_unload::<S>(), systems);
        self
    }
    /// Runs once when screen finished unloading. Shorthand for
    /// `app.add_systems(on_screen_unloaded::<S>(), systems)`.
    pub fn on_unloaded<M>(self, systems: impl IntoScheduleConfigs<ScheduleSystem, M>) -> Self {
        self.app.add_systems(on_screen_unloaded::<S>(), systems);
        self
    }

    /// Builds the schedule and adds it to the app.
    pub fn build(self) {
        // init
        let app = self.app;
        let id = app.world_mut().register_component::<S>();
        let tick = app.world_mut().change_tick();
        let mut registry = app.world_mut().get_resource_or_init::<ScreenRegistry>();

        // insert data
        let mut data = ScreenData::new::<S>(id, tick);
        let skip_load = self
            .schedules
            .get(&ScreenSchedule::Loading)
            .map(|v| v.systems_len() == 0)
            .unwrap_or_default();
        let skip_unload = self
            .schedules
            .get(&ScreenSchedule::Unloading)
            .map(|v| v.systems_len() == 0)
            .unwrap_or_default();
        data.set_skip_load(self.skip_load.unwrap_or(skip_load));
        data.set_skip_unload(self.skip_unload.unwrap_or(skip_unload));
        data.set_load_strategy(self.load_strategy);
        debug!("Built screen {data:#?}");
        registry.insert(id, data);

        // watch screen switcher
        app.add_observer(on_switch_screen::<S>);
        info!("watching on_switch_screen for {}", S::name());

        // scope systems
        for (_, schedule) in self.schedules.into_iter() {
            app.add_schedule(schedule);
        }

        // spawn on load
        app.add_systems(on_screen_load::<S>(), S::spawn);

        // Lifecycle
        #[cfg(debug_assertions)]
        {
            app.add_systems(on_screen_load::<S>(), || debug!("Loading {:?}", S::name()));
            app.add_systems(on_screen_ready::<S>(), || debug!("Ready {:?}", S::name()));
            app.add_systems(on_screen_unload::<S>(), || {
                debug!("Unloading {:?}", S::name())
            });
            app.add_systems(on_screen_unloaded::<S>(), || {
                debug!("Unloaded {:?}", S::name())
            });
        }
        app.add_systems(on_screen_unloaded::<S>(), clean_up_scoped_entities::<S>);
    }
}

fn clean_up_scoped_entities<S: Screen>(
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
    screen_scoped.iter().for_each(|e| {
        commands.entity(e).despawn();
    });
    next_state.unload();
}
