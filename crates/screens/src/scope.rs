pub use crate::prelude::*;
use bevy::{ecs::system::ScheduleSystem, platform::collections::HashMap};
use strum::IntoEnumIterator;

pub trait RegisterScreen {
    /// Registers a [Screen] to the application.
    fn register_screen<S: Screen>(&mut self) -> &mut Self;
}
impl RegisterScreen for App {
    fn register_screen<S: Screen>(&mut self) -> &mut Self {
        S::builder(ScreenScopeBuilder::<S>::new()).build(self);
        self
    }
}

// TODO: DOCUMENT ME
pub struct ScreenScopeBuilder<S>
where
    S: Screen,
{
    schedules: HashMap<ScreenSchedule, Schedule>,
    skip_load: Option<bool>,
    skip_unload: Option<bool>,
    load_strategy: LoadStrategy,
    _ghost: PhantomData<S>,
}

impl<S> ScreenScopeBuilder<S>
where
    S: Screen,
{
    pub fn new() -> Self {
        let schedules = ScreenSchedule::iter()
            .map(|kind| (kind, Schedule::new(ScreenScheduleLabel::new::<S>(kind))))
            .collect::<HashMap<_, _>>();
        Self {
            schedules,
            skip_load: None,
            skip_unload: None,
            load_strategy: LoadStrategy::default(),
            _ghost: PhantomData,
        }
    }

    /// Initialize directly into ready state. By default this is true, unless
    /// there are systems present in the Load schedule.
    pub fn with_skip_load(&mut self, val: bool) -> &mut Self {
        self.skip_load = Some(val);
        self
    }
    /// Deinitialize directly into unloaded state. By default this is true,
    /// unless there are systems present in the Unload schedule.
    pub fn with_skip_unload(&mut self, val: bool) -> &mut Self {
        self.skip_unload = Some(val);
        self
    }
    /// Sets the [LoadingStrategy]. By default, this is Blocking.
    pub fn with_load_strategy(&mut self, val: LoadStrategy) -> &mut Self {
        self.load_strategy = val;
        self
    }

    /// Add systems to the schedule scope. Will run in the specified schedule.
    ///
    /// The following schedules run on every (fixed) update:
    ///
    /// - Update
    /// - FixedUpdate
    /// - Loading
    /// - Unloading
    ///
    /// ... While the following run only on screen state transitions:
    ///
    /// - OnLoad
    /// - OnReady
    /// - OnUnload
    /// - OnUnloaded
    ///
    /// Note that adding an `On` system _will not_ automatically enable loading
    /// or unloading for this screen. Make sure you either have systems in the
    /// [ScreenSchedule::Load](ScreenSchedule) schedule, or you manually set
    /// [with_skip_load(false)](ScreenScopeBuilder::with_skip_load), or the
    /// analgous for unloading.
    pub fn add_systems<M>(
        &mut self,
        kind: ScreenSchedule,
        systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
    ) -> &mut Self {
        self.schedules
            .entry(kind)
            .or_insert(Schedule::new(ScreenScheduleLabel::new::<S>(kind)))
            .add_systems(systems);
        self
    }

    fn build(self, app: &mut App) {
        // init
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
        for (kind, schedule) in self.schedules.into_iter() {
            let label = schedule.label();
            match kind {
                ScreenSchedule::OnLoad => {
                    app.add_systems(on_screen_load::<S>(), move |mut commands: Commands| {
                        commands.run_schedule(label)
                    });
                }
                ScreenSchedule::OnReady => {
                    let label = schedule.label();
                    app.add_systems(on_screen_ready::<S>(), move |mut commands: Commands| {
                        commands.run_schedule(label)
                    });
                }
                ScreenSchedule::OnUnload => {
                    let label = schedule.label();
                    app.add_systems(on_screen_unload::<S>(), move |mut commands: Commands| {
                        commands.run_schedule(label)
                    });
                }
                ScreenSchedule::OnUnloaded => {
                    let label = schedule.label();
                    app.add_systems(on_screen_unloaded::<S>(), move |mut commands: Commands| {
                        commands.run_schedule(label)
                    });
                }
                _ => {
                    // run on update, see [run_schedules](systems.rs)
                }
            }
            app.add_schedule(schedule);
        }

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

impl<S> Default for ScreenScopeBuilder<S>
where
    S: Screen,
{
    fn default() -> Self {
        Self::new()
    }
}

fn clean_up_scoped_entities<S: Screen>(
    mut commands: Commands,
    mut screen_data: ScreenDataMut<S>,
    // Any entity which is (explicitly marked as ScreenScoped, or is _not_ marked
    // as persistent) _and_ is not a top-level observer
    screen_scoped: Query<
        Entity,
        (
            Or<(
                With<ScreenScoped>,  // is explicitly screen-scoped
                Without<Persistent>, // is explicitly persistent
            )>,
        ),
    >,
    top_levels: Query<Entity, (Or<(With<Observer>, With<Window>)>, Without<ChildOf>)>,
) {
    screen_scoped
        .iter()
        .filter(|c| !top_levels.iter().contains(c))
        .for_each(|e| {
            commands.entity(e).detach_all_children().despawn();
        });
    screen_data.unload();
}
