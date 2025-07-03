use std::{path::PathBuf, str::FromStr};

use bevy::prelude::*;
use egui::mutex::Mutex;
use egui_dock::{DockArea, Style};
use egui_file_dialog::FileDialog;

use crate::{
    scene::SaveSceneEvent,
    ui::{
        UiState,
        file_dialog::UiFileState,
        tabs::{DockState, TabViewer},
        toast::Toast,
    },
};
#[derive(Debug, Default)]
pub struct Layout;
impl Layout {
    fn top_panel(
        ui_state: &mut UiState,
        world: &mut World,
        ctx: &mut egui::Context,
    ) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Scene", |ui| {
                    if ui.button("Load Scene").clicked() {
                        ui_state.file_dialog = FileDialog::new()
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
                    ui_state,
                    "Got picked file when UiFileState was None".into(),
                ),
                UiFileState::SavingScene => {
                    debug!("sending SaveSceneEvent({path:?})");
                    world.trigger(SaveSceneEvent(path));
                }
                UiFileState::LoadingScene => {
                    Toast::Warning.from_ui_state(ui_state, "Todo!".into());
                    ui_state.file_dialog_state = UiFileState::None;
                }
                UiFileState::SavingLayout => {
                    Toast::Warning.from_ui_state(ui_state, "Todo!".into());
                    ui_state.file_dialog_state = UiFileState::None;
                }
                UiFileState::LoadingLayout => {
                    Toast::Warning.from_ui_state(ui_state, "Todo!".into());
                    ui_state.file_dialog_state = UiFileState::None;
                }
            }
        }
    }
    fn main_zone(
        ui_state: &mut UiState,
        world: &mut World,
        ctx: &mut egui::Context,
    ) {
        world.resource_scope::<DockState, _>(|world, mut dock_state| {
            DockArea::new(&mut dock_state)
                .style(Style::from_egui(ctx.style().as_ref()))
                .show(
                    ctx,
                    &mut TabViewer {
                        world,
                        state: Mutex::new(ui_state),
                    },
                );
        });
    }
    pub fn render(
        ui_state: &mut UiState,
        world: &mut World,
        ctx: &mut egui::Context,
    ) {
        Self::top_panel(ui_state, world, ctx);
        Self::main_zone(ui_state, world, ctx);
        ui_state.toasts.show(ctx);
        ui_state.file_dialog.update(ctx);
    }
}
