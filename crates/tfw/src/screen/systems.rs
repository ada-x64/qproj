//! This module contains all systems, including observers.

use crate::prelude::*;

fn on_switch_by_name(
    trigger: On<SwitchToScreenByName>,
    registry: Res<ScreenRegistry>,
    mut commands: Commands,
) {
    let name = &**trigger.event();
    if let Some(screen) = registry.iter().find(|i| i.name == *name) {
        commands.run_system(screen.trigger_load);
    } else {
        error!("Could not find screen with name {name:?}");
    }
}

/// NOTE: This is registered in scope.rs
/// Panics: This should _never_ fail to find a registered screen, so it will panic if it does so.
pub fn on_switch_screen<T: Screen>(
    _trigger: On<SwitchToScreen<T>>,
    registry: Res<ScreenRegistry>,
    mut commands: Commands,
) {
    if let Some(screen) = registry.iter().find(|i| i.name == T::name()) {
        commands.run_system(screen.trigger_load);
    } else {
        panic!("Could not find screen with name {:?}", T::name());
    }
}

pub fn plugin(app: &mut App) {
    app.add_observer(on_switch_by_name);
}
