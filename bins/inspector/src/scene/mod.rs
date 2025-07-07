use std::{any::TypeId, ffi::OsStr, path::PathBuf};

use bevy::{
    asset::AssetPath, ecs::world::CommandQueue, platform::collections::HashSet,
    prelude::*, scene::SceneLoader, tasks::IoTaskPool,
};
use q_tasks::task;
use tiny_bail::prelude::*;

use crate::{
    scene::{gizmos::GizmosPlugin, inspector_cam::InspectorCamPlugin},
    ui::modals::toast::Toast,
};

pub mod gizmos;
pub mod inspector_cam;

#[derive(Resource, Clone, Default)]
pub struct CurrentScene(Option<Handle<Scene>>);

#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum LoadStatus {
    #[default]
    Complete,
    AwaitingLoad(Handle<Scene>),
}
#[derive(States, Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum SaveStatus {
    #[default]
    Complete,
    AwaitingSave(PathBuf),
}

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::scene::ScenePlugin>() {
            app.add_plugins(bevy::scene::ScenePlugin);
        }
        app.add_plugins((InspectorCamPlugin, GizmosPlugin))
            .init_resource::<CurrentScene>()
            .init_state::<LoadStatus>()
            .init_state::<SaveStatus>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn((
                    Name::new("Scene Root"),
                    DynamicSceneRoot::default(),
                ));
            })
            .add_systems(
                PreUpdate,
                await_loaded_scene.run_if(
                    |load_status: Res<State<LoadStatus>>| {
                        matches!(**load_status, LoadStatus::AwaitingLoad(_))
                    },
                ),
            )
            .add_event::<SaveSceneEvent>()
            .add_event::<LoadSceneEvent>()
            .add_event::<ApplySceneEvent>()
            .add_observer(save_scene)
            .add_observer(load_scene)
            .add_observer(apply_scene);
    }
}

pub trait SceneCommands {
    fn trigger_scene_save(self, path: PathBuf) -> Self;
    fn trigger_scene_load(self, path: PathBuf) -> Self;
}
impl<'w, 's> SceneCommands for Commands<'w, 's> {
    fn trigger_scene_save(mut self, path: PathBuf) -> Self {
        self.trigger(SaveSceneEvent(path));
        self
    }
    fn trigger_scene_load(mut self, path: PathBuf) -> Self {
        self.trigger(LoadSceneEvent(path));
        self
    }
}

// PERF: Non-tail recrusive. Could be an issue.
fn extract<'w>(
    mut builder: DynamicSceneBuilder<'w>,
    world: &World,
    entity: Entity,
) -> DynamicSceneBuilder<'w> {
    r!(builder, world.get_entity(entity)); //check if the entity exists
    builder = builder.extract_entity(entity);
    if let Some(children) = world.entity(entity).get::<Children>() {
        for child in children {
            builder = extract(builder, world, *child);
        }
        builder
    } else {
        builder
    }
}

#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);
fn save_scene(trigger: Trigger<SaveSceneEvent>, world: &mut World) {
    let mut path = trigger.0.clone();
    r!(world.get_resource_mut::<NextState<SaveStatus>>())
        .set(SaveStatus::AwaitingSave(path.clone()));
    path.set_extension("scn.ron");
    let scene = {
        let mut scene =
            world.query_filtered::<Entity, With<DynamicSceneRoot>>();
        let scene = scene.single(world);
        if let Err(e) = scene {
            Toast::Error.from_world(world, e.to_string());
            return;
        }
        scene.unwrap()
    };
    let builder = {
        let builder = DynamicSceneBuilder::from_world(world)
            .with_resource_filter(SceneFilter::Denylist(HashSet::from_iter([
                TypeId::of::<Time>(),
            ])));
        extract(builder, world, scene)
    };
    let serialized_scene = {
        let serialized_scene = {
            let type_registry = r!(world.get_resource::<AppTypeRegistry>());
            builder.build().serialize(&type_registry.read())
        };
        if let Err(e) = serialized_scene {
            Toast::Error.from_world(world, e.to_string());
            return;
        }
        serialized_scene.unwrap()
    };

    // might take a sec so don't block
    task!(IoTaskPool, async move |q: &mut CommandQueue| {
        q.push(move |world: &mut World| {
            let res = std::fs::write(&path, serialized_scene.as_bytes());
            match res {
                Err(e) => Toast::Error.from_world(world, e.to_string()),
                Ok(_) => Toast::Success
                    .from_world(world, format!("Saved file to {path:#?}")),
            };
            world
                .resource_mut::<NextState<SaveStatus>>()
                .set(SaveStatus::Complete);
        })
    })(world);
}

#[derive(Event)]
pub struct LoadSceneEvent(pub PathBuf);
fn load_scene(trigger: Trigger<LoadSceneEvent>, world: &mut World) {
    // ensure valid file type
    let path = trigger.event().0.clone();
    let ext = path.extension().and_then(|ext| ext.to_str());
    if !matches!(ext, Some("scn") | Some("ron")) {
        Toast::Error.from_world(
            world,
            format!(
                "Not loading non-scene file {path:?} with extension {:?}",
                path.extension()
            ),
        );
        return;
    }
    let asset_server = r!(world.get_resource::<AssetServer>());
    let path = AssetPath::from_path(&path);
    let full_ext = path.get_full_extension();
    let label = path.label();
    debug!("path: {path:#?}\nfull_ext: {full_ext:#?}\nlabel: {label:#?}");
    let scene_handle = asset_server.load(path);
    let mut next_state = world.resource_mut::<NextState<LoadStatus>>();
    next_state.set(LoadStatus::AwaitingLoad(scene_handle));
}

fn await_loaded_scene(
    asset_server: Res<AssetServer>,
    state: Res<State<LoadStatus>>,
    mut next_state: ResMut<NextState<LoadStatus>>,
    mut current_scene: ResMut<CurrentScene>,
    mut commands: Commands,
) {
    match &**state {
        LoadStatus::AwaitingLoad(scene) => {
            asset_server.is_loaded(scene);
            next_state.set(LoadStatus::Complete);
            current_scene.0 = Some(scene.clone());
            commands.trigger(ApplySceneEvent);
        }
        _ => {
            unreachable!();
        }
    }
}

#[derive(Event)]
struct ApplySceneEvent;
fn apply_scene(_trigger: Trigger<ApplySceneEvent>, world: &mut World) {
    // reset world into default scene...
    let scene = world.resource::<CurrentScene>().0.clone();
    if scene.is_none() {
        Toast::Error.from_world(world, "Tried to apply empty scene!");
        return;
    }
    let id = {
        let mut root = world.query::<&mut DynamicSceneRoot>();
        let root = root.single_mut(world);
        match root {
            Ok(root) => root.id(),
            Err(_) => {
                let root = DynamicSceneRoot::default();
                let id = root.id();
                world.spawn(root);
                id
            }
        }
    };

    // clear root, assure existence
    world.resource_scope(
        |world: &mut World, mut dyscenes: Mut<Assets<DynamicScene>>| {
            let default = DynamicScene::default();
            dyscenes.insert(id, default);
            let ds = dyscenes.get_mut(id).unwrap();

            let scene = {
                let assets = r!(world.get_resource::<Assets<Scene>>());
                let scene = assets.get(scene.unwrap().id());
                if scene.is_none() {
                    Toast::Error.from_world(
                        world,
                        "Could not load scene! See logs for details.",
                    );
                    return;
                }
                scene.unwrap()
            };

            *ds = DynamicScene::from_scene(scene);
        },
    );
}
