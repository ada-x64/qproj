//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use crate::tabs::*;
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
use q_tasks::TaskComponent;

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

pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

pub fn show_toast_cmd(t: ToastType, msg: String) -> impl Command {
    move |world: &mut World| {
        let mut ui_state = world
            .get_resource_mut::<UiState>()
            .expect("Couldn't get UI state!");
        match t {
            ToastType::Success => ui_state.toasts.success(msg),
            ToastType::Error => ui_state.toasts.error(msg),
            ToastType::Warning => ui_state.toasts.warning(msg),
            ToastType::Info => ui_state.toasts.info(msg),
        };
    }
}

#[derive(Debug, Event)]
pub enum UiEvent {
    FileDialogFinished(FileType),
    FileSaveFinished(FileType),
}

#[derive(Resource, Derivative)]
#[derivative(Debug, Default)]
pub struct UiState {
    #[derivative(Default(value = "egui::Rect::NOTHING"))]
    pub viewport_rect: egui::Rect,
    #[derivative(Default(value = "InspectorSelection::Entities"))]
    pub selection: InspectorSelection,
    pub selected_entities: SelectedEntities,
    #[derivative(Debug = "ignore")]
    pub toasts: egui_notify::Toasts,
}
impl UiState {
    fn save_scene(&mut self, world: &mut World) {
        let mut component = world.spawn_empty();
        let id = component.id();
        let task = IoTaskPool::get().spawn(async move {
            let mut command_queue = CommandQueue::default();
            info!("Getting file handle...");
            let handle = rfd::AsyncFileDialog::new()
                .set_directory(std::env::current_dir().unwrap_or_default())
                .add_filter("scene", &[".scn.ron"])
                .save_file()
                .await;
            match handle {
                Some(handle) => {
                    info!("Got file handle...");
                    command_queue.push(|world: &mut World| {
                        let scene = DynamicScene::from_world(world);
                        let type_registry = world.resource::<AppTypeRegistry>();
                        let type_registry = type_registry.read();
                        let serialized_scene =
                            scene.serialize(&type_registry).unwrap();
                        info!("Serialized scene...");
                        IoTaskPool::get()
                            .spawn(async move {
                                let mut q = CommandQueue::default();
                                let path =
                                    handle.path().to_string_lossy().to_string();
                                let res = handle
                                    .write(serialized_scene.as_bytes())
                                    .await;
                                match res {
                                    Err(e) => {
                                        q.push(show_toast_cmd(
                                            ToastType::Error,
                                            e.to_string(),
                                        ));
                                    }
                                    Ok(_) => q.push(show_toast_cmd(
                                        ToastType::Success,
                                        format!("Saved file to {path}"),
                                    )),
                                }
                                q
                            })
                            .detach();
                    });
                }
                None => {
                    info!("Failed to get file handle");
                    command_queue.push(show_toast_cmd(
                        ToastType::Error,
                        "Failed to save file".into(),
                    ));
                    command_queue.push(move |world: &mut World| {
                        world.despawn(id);
                    });
                }
            }
            command_queue
        });
        component.insert(TaskComponent(task));
    }
    fn top_panel(&mut self, world: &mut World, ctx: &mut egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Scene", |ui| {
                    if ui.button("Load Scene").clicked() {}
                    if ui.button("Save Scene").clicked() {
                        self.save_scene(world);
                    }
                });
                ui.menu_button("Layout", |ui| {
                    if ui.button("Load Layout").clicked() {}
                    if ui.button("Save Layout").clicked() {}
                    if ui.button("Save Layout As...").clicked() {}
                });
            })
        });
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
