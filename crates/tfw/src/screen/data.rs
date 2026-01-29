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

/// Triggered when a [Screen] finishes unloading and is
/// ready to transition.
#[derive(Event, Debug, PartialEq, Eq, Clone, Copy)]
pub struct FinishUnload;

/// Call this when you want to switch screens.
#[derive(Event, Debug, PartialEq, Eq, Clone, Deref, Default)]
pub struct SwitchToScreen<S: Screen>(PhantomData<S>);

/// Call this when you want to switch screens but only have access to the
/// screen's name, e.g. in serialization scenarios. Whenever possible, prefer to
/// use [SwitchToScreen].
#[derive(Event, Debug, PartialEq, Eq, Clone, Deref)]
pub struct SwitchToScreenByName(pub String);

/// Marker struct for a screen.
#[derive(Component, Reflect)]
pub struct ScreenMarker(pub ComponentId);

/// Stores a map from the system's name to its spawn function.
/// Used to dynamically load a screen.
#[derive(Resource, Debug, Deref, DerefMut, Default)]
pub struct ScreenRegistry(HashMap<ComponentId, ScreenData>);

#[derive(Debug)]
pub struct ScreenData {
    pub name: String,
    pub id: ComponentId,
    pub state: ScreenStateKind,
}

/// An empty settings parameter.
#[derive(Resource, Default)]
pub struct NoSettings;

/// An empty [AssetCollection]. Combine this with the Nonblocking
/// [LoadingStrategy] to skip asset loading.
/// Note: This will _never_ resolve, so the [ScreenLoadingState] will _never_ be
/// Ready.
#[derive(Resource, Default, AssetCollection)]
pub struct NoAssets {}

/// A screen's [Schedule]. All systems added to this schedule, using the
/// [ScreenScope] below, will be scoped to this screen's lifetime. That is,
/// they will only run when the screen is in [ScreenStatus::Ready].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum ScreenScheduleKind {
    Main,
    Fixed,
    Load,
    Unload,
}
impl From<ScreenScheduleKind> for ScreenStateKind {
    fn from(value: ScreenScheduleKind) -> Self {
        match value {
            ScreenScheduleKind::Main | ScreenScheduleKind::Fixed => ScreenStateKind::Ready,
            ScreenScheduleKind::Load => ScreenStateKind::Loading,
            ScreenScheduleKind::Unload => ScreenStateKind::Unloading,
        }
    }
}

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenScheduleLabel<S: Screen> {
    _ghost: PhantomData<S>,
    kind: ScreenScheduleKind,
}
impl<S: Screen> ScreenScheduleLabel<S> {
    pub fn new(kind: ScreenScheduleKind) -> Self {
        Self {
            _ghost: PhantomData,
            kind,
        }
    }
}

/// Not to be registered as a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScreenStateKind {
    #[default]
    Unloaded,
    Loading,
    Ready,
    Unloading,
}
impl ScreenStateKind {
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

/// How should the screen load its assets?
/// If `LoadingStrategy` is Blocking, the screen's systems will not run until
/// loading is complete. If it is Nonblocking, the screen's systems will run
/// regardless of asset completion status.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoadingStrategy {
    Blocking,
    Nonblocking,
}
impl LoadingStrategy {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Blocking)
    }
}

#[derive(Deref)]
pub struct ScreenState<S: Screen> {
    #[deref]
    kind: ScreenStateKind,
    _ghost: PhantomData<S>,
}

// Safety
// The implementor must ensure the following is true.
// SystemParam::init_access correctly registers all World accesses used by SystemParam::get_param with the provided system_meta.
// None of the world accesses may conflict with any prior accesses registered on system_meta.
unsafe impl<S: Screen> SystemParam for ScreenState<S> {
    type State = ();
    type Item<'world, 'state> = Self;

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
        Self {
            _ghost: PhantomData,
            kind: data.state,
        }
    }
}
unsafe impl<S: Screen> ReadOnlySystemParam for ScreenState<S> {}

pub struct ScreenStateMut<'w, S: Screen> {
    _ghost: PhantomData<S>,
    registry: Mut<'w, ScreenRegistry>,
    cid: ComponentId,
}
impl<'w, S: Screen> ScreenStateMut<'w, S> {
    /// Loads the screen. Has no effect if the screen is already Loaded or Ready.
    pub fn load(&mut self) {
        let state = &mut self.data_mut().state;
        match state {
            ScreenStateKind::Unloaded | ScreenStateKind::Unloading => {
                *state = ScreenStateKind::Loading;
            }
            _ => {}
        }
    }
    /// Unloads the screen. Has no effect if the screen is already Unloading or Unloaded.
    pub fn unload(&mut self) {
        let state = &mut self.data_mut().state;
        match state {
            ScreenStateKind::Loading | ScreenStateKind::Ready => {
                *state = ScreenStateKind::Unloading;
            }
            _ => {}
        }
    }
    pub fn data(&self) -> &ScreenData {
        self.registry.get(&self.cid).unwrap()
    }
    fn data_mut(&mut self) -> &mut ScreenData {
        self.registry.get_mut(&self.cid).unwrap()
    }
}
unsafe impl<'w, S: Screen> SystemParam for ScreenStateMut<'w, S> {
    type State = ();
    type Item<'world, 'state> = ScreenStateMut<'world, S>;

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
        _change_tick: bevy::ecs::change_detection::Tick,
    ) -> Self::Item<'world, 'state> {
        let registry = unsafe { world.get_resource_mut::<ScreenRegistry>().unwrap() };
        let cid = world.components().get_id(TypeId::of::<S>()).unwrap();
        Self::Item {
            registry,
            cid,
            _ghost: PhantomData,
        }
    }
}

pub fn screen_has_state<S: Screen>(
    kind: ScreenStateKind,
) -> impl FnMut(ScreenState<S>) -> bool + Clone {
    move |state: ScreenState<S>| state.kind == kind
}

/// Label of a schedule which fires when the screen has begun to load.
#[derive(ScheduleLabel, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct OnScreenLoad<S: Screen>(PhantomData<S>);
pub fn on_screen_load<S: Screen>() -> OnScreenLoad<S> {
    OnScreenLoad::<S>::default()
}

/// Label of a schedule which fires when the screen has finished loading.
#[derive(ScheduleLabel, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct OnScreenReady<S: Screen>(PhantomData<S>);
pub fn on_screen_ready<S: Screen>() -> impl ScheduleLabel {
    OnScreenReady::<S>::default()
}

/// Label of a schedule which fires when the screen is beginning to unload. Not to be confused with [OnScreenUnloaded].
#[derive(ScheduleLabel, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct OnScreenUnload<S: Screen>(PhantomData<S>);
pub fn on_screen_unload<S: Screen>() -> impl ScheduleLabel {
    OnScreenUnload::<S>::default()
}

/// Label of a schedule which fires when the screen has finished unloading and is no longer active.
#[derive(ScheduleLabel, Default, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct OnScreenUnloaded<S: Screen>(PhantomData<S>);
pub fn on_screen_unloaded<S: Screen>() -> impl ScheduleLabel {
    OnScreenUnloaded::<S>::default()
}
