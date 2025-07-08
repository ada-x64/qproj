// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod dock;
pub mod top_bar;

use crate::ui::layout::{dock::DockPlugin, top_bar::TopBarPlugin};
use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct LayoutPlugin;

impl Plugin for LayoutPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DockPlugin, TopBarPlugin));
    }
}
