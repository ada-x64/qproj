// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

use crate::ui::{
    layout::{LayoutPlugin, dock::TabData},
    modals::ModalsPlugin,
    modals::file_dialog::UiFileState,
};
use bevy::prelude::*;
use derivative::Derivative;

#[derive(Resource, Derivative)]
#[derivative(Debug, Default)]
pub struct UiState {
    pub tab_data: TabData,
    #[derivative(Debug = "ignore")]
    pub toasts: egui_notify::Toasts,
    pub file_dialog: egui_file_dialog::FileDialog,
    pub file_dialog_state: UiFileState,
}

#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct UiSystems;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LayoutPlugin, ModalsPlugin))
            .init_resource::<UiState>();
    }
}
