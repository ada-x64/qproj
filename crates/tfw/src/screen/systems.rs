//! This module contains all systems, including observers.

use bevy::ecs::component::ComponentIdFor;

use crate::prelude::*;

fn on_switch_by_name(trigger: On<SwitchToScreenByName>, mut registry: ResMut<ScreenRegistry>) {
    let name = &**trigger.event();
    if let Some((_, screen)) = registry.iter_mut().find(|(_k, v)| v.name == *name) {
        screen.state = ScreenStateKind::Loading;
    } else {
        error!("Could not find screen with name {name:?}");
    }
}
/// NOTE: This is registered in scope.rs
/// Panics: This should _never_ fail to find a registered screen, so it will panic if it does so.
pub fn on_switch_screen<S: Screen>(
    _trigger: On<SwitchToScreen<S>>,
    mut registry: ResMut<ScreenRegistry>,
    id: ComponentIdFor<S>,
) {
    if let Some(screen) = registry.get_mut(&id.get()) {
        screen.state = ScreenStateKind::Loading;
    } else {
        error!("Could not find screen with name {:?}", S::name());
    }
}

pub fn plugin(app: &mut App) {
    app.add_observer(on_switch_by_name);
}
