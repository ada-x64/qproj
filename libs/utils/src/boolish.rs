// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
pub use paste::paste;

pub trait BoolishStateTrait: From<bool> {
    /// Returns the value as a boolean.
    fn as_bool(&self) -> bool;
    /// Returns the opposite value, consuming self.
    fn toggle(self) -> Self;
    fn event_from_bool(val: bool) -> impl Event;
}

/// Sets up simple boolean states with third-value for initialization.
/// Don't forget to call setup_boolish_states when setting up your states!
/// TODO: Make this a Derive proc_macro and seperate the setup_boolish macro.
#[macro_export]
macro_rules! boolish_states {
    ($($name: ident),*) => {
        $(
            $crate::paste! {
                #[derive(
                    Default, States, Debug, Copy, Clone, PartialEq, Eq, Hash, Reflect
                )]
                #[reflect(State)]
                pub enum [<$name States>] {
                    /// awaiting setup
                    #[default]
                    Init,
                    /// gloss to bool: true
                    Enabled,
                    /// gloss to bool: false
                    Disabled,
                }
                impl From<bool> for [<$name States>] {
                    fn from(value: bool) -> Self {
                        if value { Self::Enabled } else { Self::Disabled }
                    }
                }
                impl From<[<$name States>]> for bool {
                    fn from(value: [<$name States>]) -> Self {
                        matches!(value, [<$name States>]::Enabled)
                    }
                }

                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<Init $name>];

                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<$name Initialized>];

                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<Set $name Enabled>](pub bool);

                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<$name Enabled>](pub bool);

            impl q_utils::BoolishStateTrait for [<$name States>] {
                fn as_bool(&self) -> bool {
                    (*self).into()
                }
                fn toggle(self) -> Self {
                    (!self.as_bool()).into()
                }
                fn event_from_bool(val: bool) -> impl Event {
                    [<Set $name Enabled>](val)
                }
            }
            }
            )*

            // Hygeine: This _should_ effectively declare a new trait whenever it's introduced.
            // That's the intended behavior.
            trait SetupBoolishStates {
                fn setup_boolish_states(&mut self) -> &mut Self;
            }
            impl SetupBoolishStates for App {
                fn setup_boolish_states(&mut self) -> &mut Self {
                    use bevy::log::debug;
                    $crate::paste! {
                        self$(
                            .init_state::<[<$name States>]>()
                            .register_type::<[<$name States>]>()
                            .add_event::<[<Init $name>]>()
                            .add_event::<[<$name Initialized>]>()
                            .add_event::<[<Set $name Enabled>]>()
                            .add_event::<[<$name Enabled>]>()
                        )*
                    }
                }
            }
    }
}
