// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{ecs::system::IntoObserverSystem, prelude::*};
pub use paste::paste;

pub trait ServiceStates {
    /// Returns the value as a boolean.
    fn is_enabled(&self) -> bool;
    /// Returns the opposite value, consuming self.
    fn get_set_event_from_bool(val: bool) -> impl Event;
}
pub trait ServicePlugin: Plugin {
    /// Sets type restrictions for macro invocation.
    fn init_func<E: 'static, B: Bundle, N>() -> impl IntoObserverSystem<E, B, N>;
}

/// Sets up a simple service.
/// When using services, states will be automatically handled for you. Prefer
/// sending events.
///
/// Example usage:
/// ```rust, ignore
/// services!(MyService, init_func);
/// // init_func must be an observer over the service's Init event.
/// // It must call `commands.trigger(MyServiceInitialized)` when it is finished.
/// fn init_func(trigger: Trigger<InitMyService>, mut commands: Commands) {
///    commands.trigger(MyServiceInitialized);
/// }
/// impl Plugin for Foo {
///     fn build(app: &mut App) {
///         app.setup_services()
///             //...
///     }
/// }
/// ```
///
/// ## Lifecycle
/// Initialize services by calling `commands.trigger(InitMyService)`.
/// The service's plugin must then trigger `MyServiceInitialized`.
/// This will set the state to Enabled, Disabled or Failed depending on the
/// result.
///
/// The below graph looks better if you can scroll horizontally :)
///
/// ```text
/// [Uninitialized]
/// |
/// v (Init)
/// [Initializing]-|------------------|
/// |(FinishInit(Err(s)))             | (FinishInit(Ok(false)))
/// v              | (FinishInit(Ok(true)))
/// [Failed(s)]    v                  v
///         [Enabled]-(Enable(false)->[Disabled]
///               ^---(Enable(true))----/
/// ```
#[macro_export]
macro_rules! service {
    ($name: ident) => {
        $crate::paste!{
            #[allow(non_snake_case)]
            pub(super) fn [<$name _init>](_trigger: Trigger<[<Init $name>]>, mut commands: Commands) {
                commands.trigger([<$name Initialized>](Ok(true)));
            }
            $crate::service!($name, [<$name _init>]);
        }
    };
    ($name: ident, $init_path: path) => {
       $crate::service_inner!($name, $init_path);
    };
}

#[macro_export]
macro_rules! service_inner {
    ($name:ident, $init_path:path) => {
        $crate::paste! {
                #[derive(
                    Default, States, Debug, Clone, PartialEq, Eq, Hash, Reflect
                )]
                #[reflect(State)]
                pub enum [<$name States>] {
                    /// awaiting setup
                    #[default]
                    Uninitialized,
                    Initializing,
                    Failed(String),
                    Enabled,
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

                /// Use this to trigger initialization.
                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<Init $name>];

                /// This is triggered by the service once it is done initializing.
                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<$name Initialized>](pub Result<bool, String>);

                /// Use this to set the current state.
                #[derive(Event, PartialEq, Eq, Hash, Debug)]
                pub struct [<Enable $name>](pub bool);

                impl $crate::ServiceStates for [<$name States>] {
                    fn is_enabled(&self) -> bool {
                        matches!(self, Self::Enabled)
                    }
                    fn get_set_event_from_bool(val: bool) -> impl Event {
                        [<Enable $name>](val)
                    }
                }

                #[allow(non_snake_case)]
                mod [<$name _private>] {
                    use super::*;
                    use bevy::prelude::*;
                    type States = [<$name States>];
                    type Init = [<Init $name>];
                    type Initialized = [<$name Initialized>];
                    type Enable = [<Enable$name>];
                    /// Automatically moves state from "Uninitialized" to "Initializing"
                    /// when [<Init $name>] is called.
                    pub(crate) fn on_init(
                        trigger: Trigger<Init>,
                        state: Res<State<States>>,
                        mut next_state: ResMut<NextState<States>>) {
                        if matches!(**state, States::Enabled | States::Disabled) {
                            warn!("Tried to initialize already-initialized service! {state:#?}");
                            return;
                        }
                        next_state.set(States::Initializing);
                        debug!("Got {:#?}", trigger.event());
                    }

                    /// Transitions out of Initializing state.
                    /// Transitions to Enabled to Disabled based on boolean Result.
                    /// Will transition to Failed(reason) on error.
                    pub(crate) fn on_finish_init(
                            trigger: Trigger<Initialized>,
                            mut state: ResMut<NextState<States>>,
                            mut commands: Commands) {
                        match &trigger.0 {
                            Err(e) => {
                                error!("Failed to initialize! {e:?}");
                                state.set([<$name States>]::Failed(e.to_string()));
                            }
                            Ok(val) => {
                                let event = [<Enable $name>](*val);
                                debug!("Succesfully initialized! Calling {event:?}");
                                commands.trigger(event);
                            }
                        }
                    }
                    /// Sets state when [<Set $name Enabled>] is called.
                    /// Will log a warning and return if service is uninitialized.
                    pub(crate) fn on_enable(
                        trigger: Trigger<Enable>,
                        state: Res<State<States>>,
                        mut next_state: ResMut<NextState<States>>
                    ) {
                        let uninitialized = matches!(*next_state, NextState::Unchanged) && matches!(**state, States::Uninitialized);
                        let failed = matches!(*next_state, NextState::Pending(States::Failed(_))) || matches!(**state, States::Failed(_));
                        if uninitialized || failed {
                            let reason = if uninitialized {"Uninitialized"} else {"Failed"};
                            bevy::log::warn!("Refusing to change state: current or next state is {reason} Event={:#?}", trigger.event());
                            return;
                        }
                        if trigger.0 {
                            next_state.set(super::[<$name States>]::Enabled);
                        } else {
                            next_state.set(super::[<$name States>]::Disabled);
                        }
                    }
                }

                pub struct [<$name ServicePlugin>];

                impl Plugin for [<$name ServicePlugin>] {
                    fn build(&self, app: &mut App) {
                        app
                            .init_state::<[<$name States>]>()
                            .register_type::<[<$name States>]>()
                            .add_event::<[<Init $name>]>()
                            .add_event::<[<$name Initialized>]>()
                            .add_event::<[<Enable $name>]>()
                            .add_observer([<$name _private>]::on_init)
                            .add_observer([<$name _private>]::on_finish_init)
                            .add_observer([<$name _private>]::on_enable)
                            .add_observer($init_path)
                            ;
                    }
                }
        }
    }
}
