// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;

#[derive(Default, States, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(State)]
pub enum BoolishState {
    /// Awaiting setup
    #[default]
    Init,
    /// Describes the UI state where: Inspector editing is active and the game
    /// is paused.
    Enabled,
    /// Describes the UI state where: Inspector editing is inactive and the game
    /// is being played.
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
impl BoolishStateTrait for BoolishState {
    fn as_bool(&self) -> bool {
        (*self).into()
    }
    fn toggle(self) -> Self {
        (!self.as_bool()).into()
    }
}

pub trait BoolishStateTrait: From<bool> {
    /// Returns the value as a boolean.
    fn as_bool(&self) -> bool;
    /// Returns the opposite value, consuming self.
    fn toggle(self) -> Self;
}

/// Sets up simple boolean states with third-value for initialization.
/// Don't forget to call setup_boolish_states when setting up your states!
/// TODO: Make this a Derive proc_macro and seperate the setup_boolish macro.
#[macro_export]
macro_rules! boolish_states {
    ($($name: ident),*) => {
        $(
            #[derive(
                Default, States, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect
            )]
            #[reflect(State)]
            pub enum $name {
                /// awaiting setup
                #[default]
                Init,
                /// gloss to bool: true
                Enabled,
                /// gloss to bool: false
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
        )*
        q_utils::impl_boolish!($($name)*);
        q_utils::setup_boolish!($($name)*);
    }
}

/// Use this when you have an enum you manually want to derive boolish for.
/// (Would be better as proc_macro but w/e)
#[macro_export]
macro_rules! impl_boolish {
    ($($name: ident)*) => {
        $(
        impl q_utils::BoolishStateTrait for $name {
            fn as_bool(&self) -> bool {
                (*self).into()
            }
            fn toggle(self) -> Self {
                (!self.as_bool()).into()
            }
        })*
    };
}

/// Creates the `setup_boolish_states` function and trait impl for App.
/// Will log to Debug whenever the state switches between $name::Enabled and
/// $name::Disabled
#[macro_export]
macro_rules! setup_boolish {
    ($($name: ident)*) => {
        // Hygeine: This _should_ effectively declare a new trait whenever it's introduced.
        // That's the intended behavior.
        trait SetupBoolishStates {
            fn setup_boolish_states(&mut self) -> &mut Self;
        }
        impl SetupBoolishStates for App {
            fn setup_boolish_states(&mut self) -> &mut Self {
                use bevy::log::debug;
                self$(.init_state::<$name>().register_type::<$name>()
                    .add_systems(OnEnter($name::Enabled), || debug!(state_name = stringify!($name), status="ENABLED"))
                    .add_systems(OnEnter($name::Disabled), || debug!(state_name = stringify!($name), status="DISABLED"))
            )*
            }
        }
    }
}
