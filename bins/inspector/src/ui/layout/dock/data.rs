// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::any::TypeId;

use bevy::{asset::UntypedAssetId, prelude::*};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use derivative::Derivative;

#[derive(Default, Debug, Eq, PartialEq)]
pub enum InspectorSelection {
    #[default]
    Entities,
    Resource(TypeId, String),
    Asset(TypeId, String, UntypedAssetId),
}

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct TabData {
    #[derivative(Default(value = "egui::Rect::NOTHING"))]
    pub viewport_rect: egui::Rect,
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
    pub show_all_entities: bool,
}

#[derive(Debug)]
pub enum Tab {
    SceneEditor,
    Inspector,
    Hierarchy,
    Resources,
    Assets,
    NoiseEditor,
    States,
}
