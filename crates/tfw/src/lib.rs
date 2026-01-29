#![feature(register_tool)]
#![register_tool(bevy)]
#![allow(bevy::panicking_methods)]

use crate::prelude::*;

/// General utility types
pub mod data;
/// Screen implementation
pub mod screen;

pub mod prelude {
    pub use super::data::prelude::*;
    pub use super::data::*;
    pub use super::screen::prelude::*;
    #[doc(hidden)]
    pub use bevy::ecs::{lifecycle::HookContext, world::DeferredWorld};
    pub(crate) use bevy::prelude::*;
}

#[derive(Default, Resource, Reflect, Debug)]
pub struct TfwSettings {
    /// Prefer to initialize this with [Self::with_initial_screen]
    pub initial_screen_name: Option<String>,
}
impl TfwSettings {
    pub fn with_initial_screen<S: Screen>() -> Self {
        Self {
            initial_screen_name: Some(S::name()),
        }
    }
}

/// The main export plugin for TFW. `Screens` should be an enum with screen
/// names. Refer to the template documentation for more details.
/// The template parameter refers to the initial screen.
#[derive(Default, Debug)]
pub struct TfwPlugin;
impl Plugin for TfwPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(screen::plugin);
        app.init_resource::<TfwSettings>();
        app.add_systems(
            Startup,
            |mut commands: Commands, settings: Res<TfwSettings>| {
                if let Some(initial_screen) = settings.initial_screen_name.clone() {
                    commands.trigger(SwitchToScreenByName(initial_screen));
                }
            },
        );
    }
}
