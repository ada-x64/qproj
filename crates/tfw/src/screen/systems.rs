//! This module contains all systems, including observers.

use bevy::ecs::component::ComponentIdFor;

use crate::prelude::*;

fn handle_switch_msg(
    mut reader: MessageReader<SwitchToScreenMsg>,
    mut registry: ResMut<ScreenRegistry>,
) {
    for item in reader.read().collect::<Vec<_>>().iter().rev() {
        if let Some(data) = registry.get_mut(&item.0) {
            data.state = ScreenState::Loading;
            return;
        } else {
            warn!("Could not find screen with component id {:?}", item.0);
        }
    }
}
/// NOTE: This is registered in scope.rs
pub fn on_switch_screen<S: Screen>(
    _trigger: On<SwitchToScreen<S>>,
    id: ComponentIdFor<S>,
    mut commands: Commands,
) {
    commands.write_message(SwitchToScreenMsg(id.get()));
}

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, handle_switch_msg);
}
