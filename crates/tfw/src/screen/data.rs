use crate::prelude::*;
use bevy::{
    ecs::{
        component::ComponentId,
        schedule::ScheduleLabel,
        system::{ReadOnlySystemParam, SystemParam},
    },
    platform::collections::HashMap,
};
use std::{any::TypeId, marker::PhantomData};

mod general_api {
    use super::*;

    /// Call this when you want to switch screens. This will trigger a
    /// [SwitchToScreenMsg] with the screen's [ComponentId].
    #[derive(Event, Debug, PartialEq, Eq, Clone, Deref, Default)]
    pub struct SwitchToScreen<S: Screen>(PhantomData<S>);
    pub fn switch_to_screen<S: Screen>() -> SwitchToScreen<S> {
        SwitchToScreen::<S>::default()
    }

    /// Switches to the given screen by its [ComponentId]. When possible, prefer
    /// to use [SwitchToScreen] to ensure type safety. This is a [Message] so we
    /// can buffer any [SwitchToScreenMsg]s to avoid conflicts. Only the last
    /// valid [SwitchToScreenMsg] will be read.
    #[derive(Message, Debug, PartialEq, Eq, Clone, Deref)]
    pub struct SwitchToScreenMsg(pub ComponentId);

    /// Scopes an entity to the current screen. The entity will be cleaned up when
    /// the [Screens] state changes. By default, all entities _except_ those listed
    /// in the [module documentation](crate::framework::screen) are screen-scoped.
    ///
    /// Note: This is effectively used to stop the downward propagation of the
    /// [Persistent] component. Since screen scoping is the default behavior, it
    /// should not be necessary to add this component in other cases.
    #[derive(Component, Debug, Reflect, Clone, Copy, Default, PartialEq)]
    pub struct ScreenScoped;

    /// Marks an entity as screen-persistent, i.e., this entity will _not_ be
    /// automatically cleaned up when the screen changes. By default, all entites
    /// _except_ those listed in the [module
    /// documentation](crate::framework::screen) are screen-scoped.
    ///
    /// In order to mark the children of this component as Persistent, you should
    /// use the [Propagate] component.
    #[derive(Component, Debug, Reflect, Clone, Copy, Default, PartialEq)]
    pub struct Persistent;
}
pub use general_api::*;

mod screens {
    use bevy::ecs::change_detection::Tick;

    use super::*;

    /// Marker struct for a screen.
    #[derive(Component, Reflect)]
    pub struct ScreenMarker(pub ComponentId);

    /// Stores a map from the system's name to its spawn function.
    /// Used to dynamically load a screen.
    #[derive(Resource, Debug, Deref, DerefMut, Default)]
    pub struct ScreenRegistry(HashMap<ComponentId, ScreenData>);

    /// Data about a given screen. This is where all the screen's identifying information lives, including it's [ScreenState].
    #[derive(Debug)]
    pub struct ScreenData {
        /// Serialized name of the [Screen]
        name: String,
        id: ComponentId,
        state: ScreenState,
        /// TypeId of the underlying [Screen] component
        type_id: TypeId,
        /// Indicates that the state has changed and needs to run the corresponding state schedule.
        pub(crate) needs_update: bool,
        pub(crate) changed_at: Tick,
        pub(crate) initialized: bool,
        /// Should the Update schedule run even while loading?
        load_strategy: LoadStrategy,
        /// Initialize directly into Ready.
        skip_load: bool,
        /// Deinitialize immediately
        skip_unload: bool,
    }
    impl ScreenData {
        pub fn new<S: Screen>(id: ComponentId, tick: Tick) -> Self {
            Self {
                name: S::name(),
                id,
                state: ScreenState::Unloaded,
                type_id: TypeId::of::<S>(),
                needs_update: true,
                skip_load: true,
                skip_unload: true,
                load_strategy: LoadStrategy::Blocking,
                changed_at: tick,
                initialized: false,
            }
        }

        /// Loads the screen.
        /// Has no effect if already in Loading or Ready states.
        pub fn load(&mut self, tick: Tick) {
            if matches!(self.state, ScreenState::Unloaded | ScreenState::Unloading) {
                if self.skip_load {
                    self.state = ScreenState::Ready
                } else {
                    self.state = ScreenState::Loading;
                }
                self.needs_update = true;
                self.changed_at = tick;
            }
        }

        /// Unloads the screen.
        /// Has no effect if already in Unloading or Unloaded states.
        pub fn unload(&mut self, tick: Tick) {
            if matches!(self.state, ScreenState::Loading | ScreenState::Ready) {
                self.state = ScreenState::Unloading;
                self.needs_update = true;
                self.changed_at = tick;
            }
        }
        /// Finishes loading the screen.
        /// Has no effect if already in Loading or Ready states.
        pub fn finish_loading(&mut self, tick: Tick) {
            if matches!(self.state, ScreenState::Loading) {
                self.state = ScreenState::Ready;
                self.needs_update = true;
                self.changed_at = tick;
            }
        }
        /// Finishes loading the screen.
        /// Has no effect if already in Loading or Ready states.
        pub fn finish_unloading(&mut self, tick: Tick) {
            if matches!(self.state, ScreenState::Unloading) {
                self.state = ScreenState::Unloaded;
                self.needs_update = true;
                self.changed_at = tick;
            }
        }

        pub fn load_strategy(&self) -> LoadStrategy {
            self.load_strategy
        }

        pub fn skip_load(&self) -> bool {
            self.skip_load
        }

        pub fn skip_unload(&self) -> bool {
            self.skip_unload
        }

        pub fn set_skip_unload(&mut self, skip_unload: bool) {
            self.skip_unload = skip_unload;
        }

        pub fn set_skip_load(&mut self, skip_load: bool) {
            self.skip_load = skip_load;
        }

        pub fn set_load_strategy(&mut self, load_strategy: LoadStrategy) {
            self.load_strategy = load_strategy;
        }

        pub fn initialized(&self) -> bool {
            self.initialized
        }

        pub fn changed_at(&self) -> Tick {
            self.changed_at
        }

        pub fn needs_update(&self) -> bool {
            self.needs_update
        }

        pub fn type_id(&self) -> TypeId {
            self.type_id
        }

        pub fn state(&self) -> ScreenState {
            self.state
        }

        pub fn id(&self) -> ComponentId {
            self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }
    }
}
pub use screens::*;

mod schedules {
    use super::*;
    /// Describes a screen's [Schedule]. All systems added to this schedule, using the
    /// [ScreenScope] below, will be scoped to this screen's lifetime. That is,
    /// they will only run when the screen is in [ScreenStatus::Ready].
    /// To use as a schedule, wrap it with [ScreenScheduleLabel].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
    pub enum ScreenSchedule {
        Update,
        FixedUpdate,
        Loading,
        Unloading,
    }
    impl From<ScreenSchedule> for ScreenState {
        fn from(value: ScreenSchedule) -> Self {
            match value {
                ScreenSchedule::Update | ScreenSchedule::FixedUpdate => ScreenState::Ready,
                ScreenSchedule::Loading => ScreenState::Loading,
                ScreenSchedule::Unloading => ScreenState::Unloading,
            }
        }
    }

    /// Wrapper around [ScreenScheduleKind]. Needed to make schedules unique per type.
    #[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ScreenScheduleLabel {
        id: TypeId,
        kind: ScreenSchedule,
    }
    impl ScreenScheduleLabel {
        pub fn new<S: Screen>(kind: ScreenSchedule) -> Self {
            Self {
                id: TypeId::of::<S>(),
                kind,
            }
        }
        pub fn from_id(kind: ScreenSchedule, id: TypeId) -> Self {
            Self { id, kind }
        }
    }
}
pub use schedules::*;

/// Describes the current state of a screen. Not an actual [State].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScreenState {
    #[default]
    Unloaded,
    Loading,
    Ready,
    Unloading,
}
impl ScreenState {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
    pub fn is_unloading(&self) -> bool {
        matches!(self, Self::Unloading)
    }
    pub fn is_unloaded(&self) -> bool {
        matches!(self, Self::Unloaded)
    }
}

mod system_params {
    use bevy::ecs::change_detection::Tick;

    use super::*;

    /// Read-only [SystemParam] for easy access to a screen's [ScreenData]
    pub struct ScreenDataRef<'w, S: Screen> {
        data: &'w ScreenData,
        _ghost: PhantomData<S>,
    }
    impl<'w, S: Screen> ScreenDataRef<'w, S> {
        pub fn data(&self) -> &'w ScreenData {
            self.data
        }
    }

    unsafe impl<'w, S: Screen> SystemParam for ScreenDataRef<'w, S> {
        type State = ();
        type Item<'world, 'state> = ScreenDataRef<'world, S>;

        fn init_state(_world: &mut World) -> Self::State {}

        fn init_access(
            _state: &Self::State,
            _system_meta: &mut bevy::ecs::system::SystemMeta,
            component_access_set: &mut bevy::ecs::query::FilteredAccessSet,
            world: &mut World,
        ) {
            component_access_set
                .add_unfiltered_resource_read(world.resource_id::<ScreenRegistry>().unwrap());
        }

        unsafe fn get_param<'world, 'state>(
            _state: &'state mut Self::State,
            _system_meta: &bevy::ecs::system::SystemMeta,
            world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
            _change_tick: bevy::ecs::change_detection::Tick,
        ) -> Self::Item<'world, 'state> {
            let cid = world.components().get_id(TypeId::of::<S>()).unwrap();
            let registry = unsafe { world.get_resource::<ScreenRegistry>().unwrap() };
            let data = registry.get(&cid).unwrap();
            ScreenDataRef {
                _ghost: PhantomData,
                data,
            }
        }
    }
    unsafe impl<'w, S: Screen> ReadOnlySystemParam for ScreenDataRef<'w, S> {}

    /// [SystemParam] for easy mutable access to the given screen's data.
    /// All functionality happens through helper functions for API sanity.
    pub struct ScreenDataMut<'w, S: Screen> {
        _ghost: PhantomData<S>,
        registry: Mut<'w, ScreenRegistry>,
        cid: ComponentId,
        change_tick: Tick,
    }
    impl<'w, S: Screen> ScreenDataMut<'w, S> {
        /// Loads the screen. Has no effect if the screen is already Loaded or Ready.
        pub fn load(&mut self) {
            let tick = self.change_tick;
            self.data_mut().load(tick);
        }
        /// Unloads the screen. Has no effect if the screen is already Loaded or Ready.
        pub fn unload(&mut self) {
            let tick = self.change_tick;
            self.data_mut().unload(tick);
        }
        /// Loads the screen. Has no effect if the screen is not Loading.
        pub fn finish_loading(&mut self) {
            let tick = self.change_tick;
            self.data_mut().finish_loading(tick);
        }
        /// Loads the screen. Has no effect if the screen is not Loading.
        pub fn finish_unloading(&mut self) {
            let tick = self.change_tick;
            self.data_mut().finish_unloading(tick);
        }
        pub fn data(&self) -> &ScreenData {
            self.registry.get(&self.cid).unwrap()
        }
        fn data_mut(&mut self) -> &mut ScreenData {
            self.registry.get_mut(&self.cid).unwrap()
        }
    }
    unsafe impl<'w, S: Screen> SystemParam for ScreenDataMut<'w, S> {
        type State = ();
        type Item<'world, 'state> = ScreenDataMut<'world, S>;

        fn init_state(_world: &mut World) -> Self::State {}

        fn init_access(
            _state: &Self::State,
            _system_meta: &mut bevy::ecs::system::SystemMeta,
            component_access_set: &mut bevy::ecs::query::FilteredAccessSet,
            world: &mut World,
        ) {
            component_access_set
                .add_unfiltered_resource_write(world.resource_id::<ScreenRegistry>().unwrap());
        }

        unsafe fn get_param<'world, 'state>(
            _state: &'state mut Self::State,
            _system_meta: &bevy::ecs::system::SystemMeta,
            world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
            change_tick: bevy::ecs::change_detection::Tick,
        ) -> Self::Item<'world, 'state> {
            let registry = unsafe { world.get_resource_mut::<ScreenRegistry>().unwrap() };
            let cid = world.components().get_id(TypeId::of::<S>()).unwrap();
            Self::Item {
                registry,
                cid,
                _ghost: PhantomData,
                change_tick,
            }
        }
    }
}
pub use system_params::*;

mod helpers {
    pub use super::*;

    /// Condition, like [in_state], but for screens.
    pub fn screen_has_state<S: Screen>(
        state: ScreenState,
    ) -> impl FnMut(ScreenDataRef<S>) -> bool + Clone {
        move |data: ScreenDataRef<S>| data.data().state() == state
    }
    /// Is the screen still loading?
    pub fn screen_loading<S: Screen>() -> impl FnMut(ScreenDataRef<S>) -> bool + Clone {
        |data: ScreenDataRef<S>| matches!(data.data().state(), ScreenState::Loading)
    }
    /// Has the screen finished loading?
    pub fn screen_ready<S: Screen>() -> impl FnMut(ScreenDataRef<S>) -> bool + Clone {
        |data: ScreenDataRef<S>| matches!(data.data().state(), ScreenState::Ready)
    }
    /// Is the screen currently unloading?
    pub fn screen_unloading<S: Screen>() -> impl FnMut(ScreenDataRef<S>) -> bool + Clone {
        |data: ScreenDataRef<S>| matches!(data.data().state(), ScreenState::Unloading)
    }
    /// Has the screen finished unloading?
    pub fn screen_unloaded<S: Screen>() -> impl FnMut(ScreenDataRef<S>) -> bool + Clone {
        |data: ScreenDataRef<S>| matches!(data.data().state(), ScreenState::Unloaded)
    }

    /// Label of a schedule which fires when the screen has begun to load.
    #[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct OnScreenLoad(pub TypeId);
    pub fn on_screen_load<S: Screen>() -> impl ScheduleLabel {
        OnScreenLoad(TypeId::of::<S>())
    }

    /// Label of a schedule which fires when the screen has finished loading.
    #[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct OnScreenReady(pub TypeId);
    pub fn on_screen_ready<S: Screen>() -> impl ScheduleLabel {
        OnScreenReady(TypeId::of::<S>())
    }

    /// Label of a schedule which fires when the screen is beginning to unload. Not to be confused with [OnScreenUnloaded].
    #[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct OnScreenUnload(pub TypeId);
    pub fn on_screen_unload<S: Screen>() -> impl ScheduleLabel {
        OnScreenUnload(TypeId::of::<S>())
    }

    /// Label of a schedule which fires when the screen has finished unloading and is no longer active.
    #[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub struct OnScreenUnloaded(pub TypeId);
    pub fn on_screen_unloaded<S: Screen>() -> impl ScheduleLabel {
        OnScreenUnloaded(TypeId::of::<S>())
    }
}
pub use helpers::*;
