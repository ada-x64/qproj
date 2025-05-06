//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;

#[derive(Default, States, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(State)]
pub enum BoolishState {
    /// Awaiting setup
    #[default]
    Init,
    /// Describes the UI state where: Inspector editing is active and the game is paused.
    Enabled,
    /// Describes the UI state where: Inspector editing is inactive and the game is being played.
    Disabled,
}
impl From<bool> for BoolishState {
    fn from(value: bool) -> Self {
        if value { Self::Enabled } else { Self::Disabled }
    }
}
impl From<BoolishState> for bool {
    fn from(value: BoolishState) -> Self {
        matches!(value, BoolishState::Enabled)
    }
}
impl BoolishState {
    /// Returns the value as a boolean.
    pub fn as_bool(&self) -> bool {
        (*self).into()
    }
    /// Returns the opposite value, consuming self.
    pub fn toggle(self) -> Self {
        (!self.as_bool()).into()
    }
}

/// Sets up simple boolean states with third-value for initialization.
/// Don't forget to call setup_macro_states when setting up your states!
#[macro_export]
macro_rules! boolish_states {
    ($($name: ident),*) => {
        $(
            #[derive(
                Default, States, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect
            )]
            #[reflect(State)]
            pub enum $name {
                /// Awaiting setup
                #[default]
                Init,
                /// Describes the UI state where: Inspector editing is active and the game is paused.
                Enabled,
                /// Describes the UI state where: Inspector editing is inactive and the game is being played.
                Disabled,
            }
            impl From<bool> for $name {
                fn from(value: bool) -> Self {
                    if value { Self::Enabled } else { Self::Disabled }
                }
            }
            impl From<$name> for bool {
                fn from(value: $name) -> Self {
                    matches!(value, $name::Enabled)
                }
            }
            impl $name {
                /// Returns the value as a boolean.
                pub fn as_bool(&self) -> bool {
                    (*self).into()
                }
                /// Returns the opposite value, consuming self.
                pub fn toggle(self) -> Self {
                    (!self.as_bool()).into()
                }
            }
        )*
        // Hygeine: This _should_ effectively declare a new trait whenever it's introduced.
        // That's the intended behavior.
        pub trait SetupBoolishStates {
            fn setup_boolish_states(&mut self) -> &mut Self;
        }
        impl SetupBoolishStates for App {
            fn setup_boolish_states(&mut self) -> &mut Self {
                self$(.init_state::<$name>().register_type::<$name>())*
            }
        }
    }
}
