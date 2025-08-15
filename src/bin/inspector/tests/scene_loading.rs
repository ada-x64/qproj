// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

use std::{
    any::{Any, TypeId},
    path::PathBuf,
};

use bevy::{prelude::*, scene::SceneInstance};
use q_inspector::{
    prelude::*,
    scene::serialize::{LoadSceneEvent, LoadStatus, SaveSceneEvent, SaveStatus},
};
use q_service::prelude::*;
use q_test::Runner;

#[derive(Resource, Default)]
struct SavedStructure(Option<DynamicScene>);

const TEST_FILE: &str = "test1.scn";
const SCENES_DIR: &str = "scenes/";
const ASSET_DIR: &str = "assets/";

#[test]
fn load_scene() -> AppExit {
    Runner::new(|app| {
        app.add_plugins(q_inspector::InspectorPlugin);
        app.finish();
        app.cleanup();
        app.init_resource::<SavedStructure>()
            .add_observer(
                |_trigger: Trigger<ServiceUp<InspectorService>>,
                 mut commands: Commands,
                 mut query: Single<(Entity, &DynamicSceneRoot, &mut Children)>,
                 scene_instances: Query<&SceneInstance>,
                 dyscenes: Res<Assets<DynamicScene>>,
                 type_registry: Res<AppTypeRegistry>,
                 mut store: ResMut<SavedStructure>| {
                    info!("1");

                    // check if ready
                    let (entity, root, _) = *query;
                    let children = query.2.as_mut();
                    let instance = children.iter().find_map(|c| {
                        Some(
                            scene_instances.get(c).ok()?.type_id() == TypeId::of::<SceneInstance>(),
                        )
                    });
                    if !instance.unwrap_or_default() {
                        info!("Failed to get SceneInstance. Trying again.");
                        return;
                    }
                    // add children
                    let handle = root.0.clone();
                    commands.entity(entity).with_children(|s| {
                        s.spawn(Name::new("a"));
                        s.spawn(Name::new("b"));
                        s.spawn(Name::new("c"));
                    });

                    // store updated scene
                    let scene = dyscenes.get(handle.id()).unwrap();
                    let scene = Scene::from_dynamic_scene(scene, type_registry.as_ref()).unwrap();
                    let scene = DynamicScene::from_scene(&scene);
                    store.0 = Some(scene);

                    // serialize
                    let full_path: PathBuf = PathBuf::from_iter([ASSET_DIR, SCENES_DIR, TEST_FILE]);
                    let full_dirname: PathBuf = PathBuf::from_iter([ASSET_DIR, SCENES_DIR]);
                    let _ = std::fs::create_dir_all(full_dirname);
                    commands.trigger(SaveSceneEvent(full_path));
                },
            )
            .add_systems(OnEnter(SaveStatus::Complete), |mut commands: Commands| {
                let rel_path: PathBuf = PathBuf::from_iter([SCENES_DIR, TEST_FILE]);
                info!("2");
                commands.trigger(LoadSceneEvent(rel_path));
            })
            .add_systems(
                OnEnter(LoadStatus::Complete),
                |query: Query<&DynamicSceneRoot>, dyscenes: Res<Assets<DynamicScene>>| {
                    info!("3");
                    let scene = &query.single().unwrap().0;
                    let scene = dyscenes.get(scene).unwrap();
                    let entities = &scene
                        .entities
                        .iter()
                        .map(|e| &e.components)
                        .collect::<Vec<_>>();
                    let resources = &scene.resources;
                    info!("scene: {{\nentities: {entities:#?}\nresources: {resources:#?}\n}}");
                },
            );
        app.run()
    })
    .run()
}
