pub use crate::prelude::*;

/// An empty settings parameter.
#[derive(Resource, Default)]
pub struct NoSettings;

/// How should the screen load its assets?
/// If `LoadingStrategy` is Blocking, the screen's systems will not run until
/// loading is complete. If it is Nonblocking, the screen's systems will run
/// regardless of asset completion status.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Reflect)]
pub enum LoadStrategy {
    #[default]
    Blocking,
    Nonblocking,
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
    /// The screen's public name. Used for serialization. Defaults to the short type path.
    fn name() -> String {
        let default = Self::default();
        Reflect::as_reflect(&default)
            .reflect_short_type_path()
            .to_owned()
    }

    /// Get the builder for this screen. Used to scope systems.
    /// Don't forget to register your screen! [App::register_screen]
    fn builder(builder: ScreenScopeBuilder<Self>) -> ScreenScopeBuilder<Self>;
}
