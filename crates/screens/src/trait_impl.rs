pub use crate::prelude::*;
use bevy::ecs::component::ComponentIdFor;

/// An empty settings parameter.
#[derive(Resource, Default)]
pub struct NoSettings;

/// How should the screen load its assets?
/// If `LoadingStrategy` is Blocking, the screen's systems will not run until
/// loading is complete. If it is Nonblocking, the screen's systems will run
/// regardless of asset completion status.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoadStrategy {
    #[default]
    Blocking,
    Nonblocking,
}
impl LoadStrategy {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Blocking)
    }
}

/// Implementation trait for Screen components.
pub trait Screen:
    Component
    + Sized
    + Default
    + Reflect
    + std::fmt::Debug
    + Clone
    + Copy
    + Eq
    + std::hash::Hash
    + Send
    + Sync
    + 'static
{
    fn name() -> String {
        let default = Self::default();
        Reflect::as_reflect(&default)
            .reflect_short_type_path()
            .to_owned()
    }

    /// Gets the spawn function. This is called when state is set to ScreenState::Loading.
    fn spawn(mut commands: Commands, id: ComponentIdFor<Self>) {
        debug!("Spawn ({})", Self::name());
        commands.spawn((Self::default(), Name::new(Self::name()), ScreenMarker(*id)));
    }
}
