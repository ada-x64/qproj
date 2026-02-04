//! This module contains all systems, including observers.

use crate::prelude::*;
use bevy::ecs::{component::ComponentIdFor, system::SystemChangeTick};

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
pub(crate) fn on_switch_screen<S: Screen>(
    _trigger: On<SwitchToScreen<S>>,
    id: ComponentIdFor<S>,
    mut commands: Commands,
) {
    commands.write_message(SwitchToScreenMsg(id.get()));
}

pub(crate) fn on_finish_loading<S: Screen>(
    _trigger: On<FinishLoading<S>>,
    mut data: ScreenDataMut<S>,
) {
    data.finish_loading();
}
pub(crate) fn on_finish_unloading<S: Screen>(
    _trigger: On<FinishUnloading<S>>,
    mut data: ScreenDataMut<S>,
) {
    data.finish_unloading();
}

fn run_schedules(
    mut registry: ResMut<ScreenRegistry>,
    mut commands: Commands,
    tick: SystemChangeTick,
    screens: Query<&ScreenMarker>,
) {
    for data in registry.values_mut() {
        if matches!(data.state(), ScreenState::Loading | ScreenState::Ready)
            && !screens.iter().contains(&ScreenMarker(data.id()))
        {
            commands.spawn((ScreenMarker(data.id()), Name::new(data.name().to_owned())));
        }
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

pub(crate) fn initial_screen(
    mut commands: Commands,
    initial_screen: Res<InitialScreen>,
    registry: Res<ScreenRegistry>,
) {
    if let Some(initial_screen) = (*initial_screen).as_ref() {
        if let Some(cid) = registry
            .values()
            .find_map(|v| (v.name() == initial_screen).then_some(v.id()))
        {
            info!("Switching to initial screen {}", *initial_screen);
            commands.write_message(SwitchToScreenMsg(cid));
        } else {
            warn!("Could not find screen with name {initial_screen}");
        }
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, initial_screen);
    app.add_systems(PostUpdate, handle_switch_msg);
    app.add_systems(Update, run_schedules);
    app.add_systems(FixedUpdate, run_fixed_schedules);
}
