use bevy::{ecs::world::CommandQueue, prelude::*};
use q_utils::text::TextUtils;
use std::time::Duration;

use crate::ui::UiState;

pub enum Toast {
    Success,
    Error,
    Warning,
    Info,
}
impl Toast {
    pub fn from_ui_state<'a>(self, ui_state: &'a mut UiState, mut msg: String) {
        let duration =
            Duration::from_secs_f32((msg.len() as f32 / 10.).max(1.));

        match self {
            Self::Success => {
                debug!(msg);
                ui_state
                    .toasts
                    .success(msg.wrap_text(80, 5).clone())
                    .duration(Some(duration))
                    .closable(true)
            }
            Self::Error => {
                error!(msg);
                ui_state
                    .toasts
                    .error(msg.wrap_text(80, 5).clone())
                    .duration(Some(duration))
                    .closable(true)
            }
            Self::Warning => {
                warn!(msg);
                ui_state
                    .toasts
                    .warning(msg.wrap_text(80, 5).clone())
                    .duration(Some(duration))
                    .closable(true)
            }
            Self::Info => {
                info!(msg);
                ui_state
                    .toasts
                    .info(msg.wrap_text(80, 5).clone())
                    .duration(Some(duration))
                    .closable(true)
            }
        };
    }
    pub fn from_world<'a>(self, world: &'a mut World, msg: String) {
        let mut ui_state = world
            .get_resource_mut::<UiState>()
            .expect("Couldn't get UI state!");
        self.from_ui_state(ui_state.as_mut(), msg);
    }
    pub fn from_queue<'a>(self, q: &'a mut CommandQueue, msg: String) {
        q.push(move |world: &mut World| {
            self.from_world(world, msg);
        })
    }
}
