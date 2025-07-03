//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---

use crate::{
    tabs::{hierarchy::HierarchyState, *},
    widgets::toast::Toast,
};
use bevy::{
    ecs::world::CommandQueue, prelude::*, tasks::IoTaskPool,
    window::PrimaryWindow,
};
use bevy_egui::{
    EguiContext, EguiPostUpdateSet,
    egui::{self, mutex::Mutex},
};
use bevy_inspector_egui::bevy_inspector::hierarchy::SelectedEntities;
use derivative::Derivative;
use egui_dock::{DockArea, NodeIndex, Style};
use egui_file_dialog::FileDialog;
use q_tasks::task;
use std::{path::PathBuf, str::FromStr};

use super::UiSystems;

// Resources //////////////////////////////////////////////////////////////////

#[derive(Resource, Deref, DerefMut)]
pub struct DockState(egui_dock::DockState<Tab>);
impl DockState {
    pub fn new(tabs: Vec<Tab>) -> Self {
        Self(egui_dock::DockState::new(tabs))
    }
}
impl Default for DockState {
    fn default() -> Self {
        // TODO ? Load layout from disk
        // Set up dock tree.
        let mut dock_state =
            DockState::new(vec![Tab::GameView, Tab::NoiseEditor]);
        let tree = dock_state.main_surface_mut();
        let [_game, _inspector] =
            tree.split_right(NodeIndex::root(), 0.75, vec![Tab::Inspector]);
        let [_game, _heirarchy] =
            tree.split_left(NodeIndex::root(), 0.25, vec![Tab::Hierarchy]);
        let [_game, _bottom] = tree.split_below(
            NodeIndex::root(),
            0.8,
            vec![Tab::Resources, Tab::Assets, Tab::States],
        );
        dock_state
    }
}

#[derive(Debug)]
pub enum FileType {
    SaveScene,
    LoadScene,
    SaveLayout,
    LoadLayout,
}

#[derive(Debug, Default)]
pub enum UiFileState {
    #[default]
    None,
    SavingScene,
    LoadingScene,
    SavingLayout,
    LoadingLayout,
}

#[derive(Resource, Derivative)]
#[derivative(Debug, Default)]
pub struct UiState {
    pub hierarchy: HierarchyState,
    #[derivative(Default(value = "egui::Rect::NOTHING"))]
    pub viewport_rect: egui::Rect,
    #[derivative(Default(value = "InspectorSelection::Entities"))]
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
    pub show_all_entities: bool,
    #[derivative(Debug = "ignore")]
    pub toasts: egui_notify::Toasts,
    pub file_dialog: egui_file_dialog::FileDialog,
    pub file_dialog_state: UiFileState,
}

impl UiState {
    fn save_scene(&mut self, world: &mut World, path: PathBuf) {
        debug!("save_scene");
        task!(IoTaskPool, async move |q: &mut CommandQueue| {
            q.push(|world: &mut World| {
                debug!("Serializing scene...");
                // TODO: This should serialize the state of the game app's
                // current scene.
                let scene = DynamicScene::from_world(world);
                let serialized_scene = {
                    let type_registry = world.resource::<AppTypeRegistry>();
                    let type_registry = type_registry.read();
                    scene.serialize(&type_registry)
                };
                task!(IoTaskPool, async move |q: &mut CommandQueue| {
                    if let Err(e) = serialized_scene {
                        Toast::Error.from_queue(q, e.to_string());
                        return;
                    }
                    let serialized_scene = serialized_scene.unwrap();
                    debug!("Saving scene to {path:?}");
                    let res =
                        std::fs::write(&path, serialized_scene.as_bytes());
                    match res {
                        Err(e) => Toast::Error.from_queue(q, e.to_string()),
                        Ok(_) => Toast::Success
                            .from_queue(q, format!("Saved file to {path:#?}")),
                    }
                })(world);
            });
        })(world)
    }
    fn top_panel(&mut self, world: &mut World, ctx: &mut egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Scene", |ui| {
                    if ui.button("Load Scene").clicked() {
                        self.file_dialog = FileDialog::new()
                            .add_file_filter_extensions(
                                "Scene File",
                                vec!["scn.ron", "scn", "ron"],
                            )
                            .initial_directory(
                                PathBuf::from_str("./assets/scenes").unwrap(),
                            );
                        self.file_dialog.pick_file();
                        self.file_dialog_state = UiFileState::LoadingScene;
                        ui.close_menu();
                    }
                    if ui.button("Save Scene").clicked() {
                        self.file_dialog = FileDialog::new()
                            .add_save_extension("Scene File", "scn.ron")
                            .allow_file_overwrite(true)
                            .initial_directory(
                                PathBuf::from_str("./assets/scenes").unwrap(),
                            );
                        self.file_dialog.save_file();
                        self.file_dialog_state = UiFileState::SavingScene;
                        ui.close_menu();
                    }
                });
                ui.menu_button("Layout", |ui| {
                    if ui.button("Load Layout").clicked() {
                        self.file_dialog = FileDialog::new()
                            .add_file_filter_extensions(
                                "Layout File",
                                vec!["layout.ron", "layout", "ron"],
                            )
                            .initial_directory(
                                PathBuf::from_str("./assets/inspector/layouts")
                                    .unwrap(),
                            );
                        self.file_dialog.pick_file();
                        self.file_dialog_state = UiFileState::LoadingScene;
                        ui.close_menu();
                    }
                    if ui.button("Save Layout").clicked() {
                        self.file_dialog = FileDialog::new()
                            .add_save_extension("Layout File", "layout.ron")
                            .allow_file_overwrite(true)
                            .initial_directory(
                                PathBuf::from_str("./assets/inspector/layouts")
                                    .unwrap(),
                            );
                        self.file_dialog.save_file();
                        self.file_dialog_state = UiFileState::SavingScene;
                        ui.close_menu();
                    }
                });
            })
        });
        // file dialog handles
        if let Some(path) = self.file_dialog.take_picked() {
            match self.file_dialog_state {
                UiFileState::None => Toast::Error.from_ui_state(
                    self,
                    "Got picked file when UiFileState was None".into(),
                ),
                UiFileState::SavingScene => {
                    self.save_scene(world, path);
                }
                UiFileState::LoadingScene => {
                    Toast::Warning.from_ui_state(self, "Todo!".into());
                    self.file_dialog_state = UiFileState::None;
                }
                UiFileState::SavingLayout => {
                    Toast::Warning.from_ui_state(self, "Todo!".into());
                    self.file_dialog_state = UiFileState::None;
                }
                UiFileState::LoadingLayout => {
                    Toast::Warning.from_ui_state(self, "Todo!".into());
                    self.file_dialog_state = UiFileState::None;
                }
            }
        }
    }
    fn main_zone(&mut self, world: &mut World, ctx: &mut egui::Context) {
        world.resource_scope::<DockState, _>(|world, mut dock_state| {
            DockArea::new(&mut dock_state.0)
                .style(Style::from_egui(ctx.style().as_ref()))
                .show(
                    ctx,
                    &mut TabViewer {
                        world,
                        state: Mutex::new(self),
                    },
                );
        });
    }
    pub fn ui(&mut self, world: &mut World, ctx: &mut egui::Context) {
        self.top_panel(world, ctx);
        self.main_zone(world, ctx);
        self.toasts.show(ctx);
        self.file_dialog.update(ctx);
    }
}

// Plugin /////////////////////////////////////////////////////////////////////
pub struct UiStatePlugin;
impl UiStatePlugin {
    pub fn show_ui_system(world: &mut World) {
        let egui_context = world
            .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
            .single(world);
        if egui_context.is_err() {
            warn!("No window.");
            return;
        }
        let mut egui_context = egui_context.unwrap().clone();

        world.resource_scope::<UiState, _>(|world, mut ui_state| {
            ui_state.ui(world, egui_context.get_mut())
        });
    }
}
impl Plugin for UiStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DockState>()
            .init_resource::<UiState>()
            .add_systems(
                PostUpdate,
                (Self::show_ui_system
                    .before(EguiPostUpdateSet::ProcessOutput)
                    .before(bevy_egui::end_pass_system)
                    .before(
                        bevy::transform::TransformSystem::TransformPropagate,
                    ))
                .in_set(UiSystems),
            );
    }
}
