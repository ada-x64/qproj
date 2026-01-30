//! This module contains all systems, including observers.

use bevy::ecs::{component::ComponentIdFor, system::SystemChangeTick};

use crate::prelude::*;

fn handle_switch_msg(
    mut reader: MessageReader<SwitchToScreenMsg>,
    mut registry: ResMut<ScreenRegistry>,
    tick: SystemChangeTick,
) {
    // get the most recent valid message, then load it and unload all others
    if reader.is_empty() {
        return;
    }
    let vec = reader.read().collect::<Vec<_>>();
    let msg_key = vec.iter().rev().find(|msg| registry.get(&msg.0).is_some());
    let msg_key = rq!(msg_key);
    for (key, data) in registry.iter_mut() {
        if *key == ***msg_key {
            data.load(tick.this_run());
        } else {
            data.unload(tick.this_run());
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

fn run_schedules(
    mut registry: ResMut<ScreenRegistry>,
    mut commands: Commands,
    tick: SystemChangeTick,
) {
    for data in registry.values_mut() {
        match data.state() {
            ScreenState::Unloaded => {
                if !data.initialized {
                    data.initialized = true;
                    data.needs_update = false;
                    data.changed_at = tick.this_run();
                }
                if data.needs_update {
                    commands.run_schedule(OnScreenUnloaded(data.type_id()));
                    data.needs_update = false;
                    data.changed_at = tick.this_run();
                }
            }
            ScreenState::Loading => {
                commands.run_schedule(ScreenScheduleLabel::from_id(
                    ScreenSchedule::Loading,
                    data.type_id(),
                ));
                if data.needs_update {
                    commands.run_schedule(OnScreenLoad(data.type_id()));
                    data.needs_update = false;
                    data.changed_at = tick.this_run();
                }
                if matches!(data.load_strategy(), LoadStrategy::Nonblocking) {
                    commands.run_schedule(ScreenScheduleLabel::from_id(
                        ScreenSchedule::Update,
                        data.type_id(),
                    ));
                }
            }
            ScreenState::Ready => {
                commands.run_schedule(ScreenScheduleLabel::from_id(
                    ScreenSchedule::Update,
                    data.type_id(),
                ));
                if data.needs_update {
                    commands.run_schedule(OnScreenReady(data.type_id()));
                    data.needs_update = false;
                    data.changed_at = tick.this_run();
                }
            }
            ScreenState::Unloading => {
                commands.run_schedule(ScreenScheduleLabel::from_id(
                    ScreenSchedule::Unloading,
                    data.type_id(),
                ));
                if data.needs_update {
                    commands.run_schedule(OnScreenUnload(data.type_id()));
                    data.needs_update = false;
                    data.changed_at = tick.this_run();
                }
            }
        }
    }
}

fn run_fixed_schedules(registry: ResMut<ScreenRegistry>, mut commands: Commands) {
    for data in registry.values() {
        match data.state() {
            ScreenState::Loading => {
                if matches!(data.load_strategy(), LoadStrategy::Nonblocking) {
                    commands.run_schedule(ScreenScheduleLabel::from_id(
                        ScreenSchedule::FixedUpdate,
                        data.type_id(),
                    ));
                }
            }
            ScreenState::Ready => {
                commands.run_schedule(ScreenScheduleLabel::from_id(
                    ScreenSchedule::FixedUpdate,
                    data.type_id(),
                ));
            }
            _ => {}
        }
    }
}

pub fn plugin(app: &mut App) {
    app.add_systems(PostUpdate, handle_switch_msg);
    app.add_systems(Update, run_schedules);
    app.add_systems(FixedUpdate, run_fixed_schedules);
}
