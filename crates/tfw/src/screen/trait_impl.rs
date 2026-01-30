use std::any::TypeId;

use bevy::ecs::component::ComponentIdFor;

pub use crate::prelude::*;

/// An empty settings parameter.
#[derive(Resource, Default)]
pub struct NoSettings;

/// An empty [AssetCollection]. Combine this with the Nonblocking
/// [LoadingStrategy] to skip asset loading.
/// Note: This will _never_ resolve, so the [ScreenLoadingState] will _never_ be
/// Ready.
#[derive(Resource, Default, AssetCollection)]
pub struct NoAssets {}

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
    /// The associated settings type. Set as [NoSettings] for no settings.
    type SETTINGS: Resource + FromWorld;
    /// Any associated assets which will load before the screen is considered
    /// ready. Use [NoAssets] to skip loading.
    /// If you want to load in assets without blocking the scoped systems,
    /// you should include asset collections and states within a service.
    type ASSETS: AssetCollection;
    /// [LoadingStrategy] for the [Screen].
    const STRATEGY: LoadingStrategy = LoadingStrategy::Nonblocking;

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

    fn has_assets() -> bool {
        TypeId::of::<Self::ASSETS>() != TypeId::of::<NoAssets>()
    }
}
