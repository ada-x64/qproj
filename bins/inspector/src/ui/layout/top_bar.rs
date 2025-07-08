// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{path::PathBuf, str::FromStr};

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext, EguiContextPass};
use egui_file_dialog::FileDialog;

use crate::{
    scene::SceneCommands,
    ui::{
        UiState, UiSystems,
        modals::{file_dialog::UiFileState, toast::Toast},
    },
};

#[derive(Default, Debug)]
pub struct TopBarPlugin;
impl Plugin for TopBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiContextPass, render.in_set(UiSystems));
    }
}

pub fn render(
    mut ui_state: ResMut<UiState>,
    mut set: ParamSet<(
        Single<&Window, With<PrimaryWindow>>,
        Single<&mut EguiContext, With<PrimaryWindow>>,
    )>,
    commands: Commands,
) {
    let winrect = set.p0().size();
    let pickerrect = egui::Vec2::new(650.0, 370.0);
    let default_pos = egui::pos2(
        winrect.x / 2. - pickerrect.x / 2.,
        winrect.y / 2. - pickerrect.y / 2.,
    );
    let mut ctx = set.p1();
    let ctx = ctx.get_mut();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("Scene", |ui| {
                if ui.button("Load Scene").clicked() {
                    ui_state.file_dialog = FileDialog::new()
                        .default_pos(default_pos)
                        .add_file_filter_extensions(
                            "Scene File",
                            vec!["scn.ron", "scn", "ron"],
                        )
                        .initial_directory(
                            PathBuf::from_str("./assets/scenes").unwrap(),
                        );
                    ui_state.file_dialog.pick_file();
                    ui_state.file_dialog_state = UiFileState::LoadingScene;
                    ui.close_menu();
                }
                if ui.button("Save Scene").clicked() {
                    ui_state.file_dialog = FileDialog::new()
                        .add_save_extension("Scene File", "scn.ron")
                        .default_pos(default_pos)
                        .allow_file_overwrite(true)
                        .initial_directory(
                            PathBuf::from_str("./assets/scenes").unwrap(),
                        );
                    ui_state.file_dialog.save_file();
                    ui_state.file_dialog_state = UiFileState::SavingScene;
                    ui.close_menu();
                }
            });
            ui.menu_button("Layout", |ui| {
                if ui.button("Load Layout").clicked() {
                    ui_state.file_dialog = FileDialog::new()
                        .add_file_filter_extensions(
                            "Layout File",
                            vec!["layout.ron", "layout", "ron"],
                        )
                        .default_pos(default_pos)
                        .initial_directory(
                            PathBuf::from_str("./assets/inspector/layouts")
                                .unwrap(),
                        );
                    ui_state.file_dialog.pick_file();
                    ui_state.file_dialog_state = UiFileState::LoadingScene;
                    ui.close_menu();
                }
                if ui.button("Save Layout").clicked() {
                    ui_state.file_dialog = FileDialog::new()
                        .add_save_extension("Layout File", "layout.ron")
                        .default_pos(default_pos)
                        .allow_file_overwrite(true)
                        .initial_directory(
                            PathBuf::from_str("./assets/inspector/layouts")
                                .unwrap(),
                        );
                    ui_state.file_dialog.save_file();
                    ui_state.file_dialog_state = UiFileState::SavingScene;
                    ui.close_menu();
                }
            });
        })
    });
    // file dialog handles
    if let Some(path) = ui_state.file_dialog.take_picked() {
        match ui_state.file_dialog_state {
            UiFileState::None => Toast::Error.from_ui_state(
                ui_state.as_mut(),
                "Got picked file when UiFileState was None",
            ),
            UiFileState::SavingScene => {
                commands.trigger_scene_save(path);
            }
            UiFileState::LoadingScene => {
                commands.trigger_scene_load(path);
            }
            UiFileState::SavingLayout => {
                Toast::Warning.from_ui_state(ui_state.as_mut(), "Todo!");
                ui_state.file_dialog_state = UiFileState::None;
            }
            UiFileState::LoadingLayout => {
                Toast::Warning.from_ui_state(ui_state.as_mut(), "Todo!");
                ui_state.file_dialog_state = UiFileState::None;
            }
        }
    }
}
