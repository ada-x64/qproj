use bevy::prelude::*;
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
    /// This is the most direct application. Prefer it when possible!
    pub fn from_ui_state(self, ui_state: &mut UiState, msg: impl ToString) {
        let mut msg = msg.to_string();
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
    pub fn from_world(self, world: &mut World, msg: impl ToString) {
        let mut ui_state = world
            .get_resource_mut::<UiState>()
            .expect("Couldn't get UI state!");
        self.from_ui_state(ui_state.as_mut(), msg);
    }
}
