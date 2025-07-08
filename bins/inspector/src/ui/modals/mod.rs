// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiContextPass};

use crate::ui::{UiState, UiSystems};

pub mod file_dialog;
pub mod toast;

pub struct ModalsPlugin;
impl Plugin for ModalsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiContextPass, render.in_set(UiSystems));
    }
}

fn render(
    mut ui_state: ResMut<UiState>,
    mut ctx: Single<&mut EguiContext, With<PrimaryWindow>>,
) {
    ui_state.toasts.show(ctx.get_mut());
    ui_state.file_dialog.update(ctx.get_mut());
}
