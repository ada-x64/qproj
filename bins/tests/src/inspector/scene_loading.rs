// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

use bevy::prelude::*;
use q_inspector::prelude::{
    InitInspector, InspectorStates,
    serialize::{LoadSceneEvent, LoadStatus, SaveSceneEvent, SaveStatus},
};

use crate::Runner;

#[derive(Resource, Default)]
struct SavedStructure(Option<DynamicScene>);

const TEST_FILE: &str = "test1.scn";
const SCENES_DIR: &str = "scenes/";
const ASSET_DIR: &str = "assets/";

#[test]
fn load_scene() -> AppExit {
    Runner::new(|app| {
        app.add_plugins(q_inspector::InspectorPlugin)
            .init_resource::<SavedStructure>()
            .add_systems(Startup, |mut commands: Commands| {
                commands.trigger(InitInspector);
                info!("Awaiting inspector initialization.");
            })
            .add_systems(
                PostUpdate,
                step1.run_if(
                    in_state(InspectorStates::Enabled)
                        .and(|s: Res<SavedStructure>| s.0.is_none()),
                ),
            )
            .add_systems(OnEnter(SaveStatus::Complete), step2)
            .add_systems(OnEnter(LoadStatus::Complete), step3);
        app.run()
    })
    .run()
}

fn step1(world: &mut World) {
    info!("Inspector enabled. Spawning world.");
    // check if ready. children dep ensures that scene instance exists.
    let mut query = world.query::<(Entity, &DynamicSceneRoot)>();
    let (entity, root) = query.single_mut(world).unwrap();
    // add children
    let handle = root.0.clone();
    world.commands().entity(entity).with_children(|s| {
        s.spawn(Name::new("a"));
        s.spawn(Name::new("b"));
        s.spawn(Name::new("c"));
    });
    world.flush();

    // store updated scene
    world.resource_scope(
        |world: &mut World, dyscenes: Mut<Assets<DynamicScene>>| {
            let scene = dyscenes.get(handle.id()).unwrap();
            let scene = {
                let type_registry =
                    world.get_resource::<AppTypeRegistry>().unwrap();
                Scene::from_dynamic_scene(scene, type_registry).unwrap()
            };
            let scene = DynamicScene::from_scene(&scene);
            {
                let mut store = world.resource_mut::<SavedStructure>();
                store.0 = Some(scene);
            }

            // serialize
            let full_path: PathBuf =
                PathBuf::from_iter([ASSET_DIR, SCENES_DIR, TEST_FILE]);
            let full_dirname: PathBuf =
                PathBuf::from_iter([ASSET_DIR, SCENES_DIR]);
            let _ = std::fs::create_dir_all(full_dirname);
            world.trigger(SaveSceneEvent(full_path));
        },
    )
}

fn step2(mut commands: Commands) {
    info!("Triggering load scene.");
    let rel_path: PathBuf = PathBuf::from_iter([SCENES_DIR, TEST_FILE]);
    commands.trigger(LoadSceneEvent(rel_path));
}

fn step3(query: Query<&DynamicSceneRoot>, dyscenes: Res<Assets<DynamicScene>>) {
    info!("Checking loaded scene");
    let scene = &query.single().unwrap().0;
    let scene = dyscenes.get(scene).unwrap();
    let entities = &scene
        .entities
        .iter()
        .map(|e| &e.components)
        .collect::<Vec<_>>();
    let resources = &scene.resources;
    info!("scene: {{\nentities: {entities:#?}\nresources: {resources:#?}\n}}");
}
