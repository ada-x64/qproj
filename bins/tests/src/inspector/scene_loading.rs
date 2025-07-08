// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::path::PathBuf;

use bevy::prelude::*;
use q_inspector::{
    scene::{LoadSceneEvent, LoadStatus, SaveSceneEvent, SaveStatus},
    state::InspectorState,
};

use crate::Runner;

#[derive(States, Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
enum TestState {
    #[default]
    Init,
    AwaitingSave,
    AwaitingLoad,
}

#[derive(Resource, Default)]
struct SavedStructure(Option<DynamicScene>);

const ASSET_PATH: &str = "assets/scene/test1.scn.ron";
const ASSET_DIR: &str = "assets/scene/";

#[test]
fn load_scene() -> AppExit {
    Runner::new(|app| {
        app.add_plugins(q_inspector::InspectorPlugin);
        app.finish();
        app.cleanup();
        app.init_state::<TestState>()
            .init_resource::<SavedStructure>()
            .add_systems(Startup, |mut state: ResMut<NextState<InspectorState>>| {
                state.set(InspectorState::Enabled);
            })
            .add_systems(OnEnter(InspectorState::Enabled),
                |mut commands: Commands,
                query: Query<Entity, With<DynamicSceneRoot>>,
                mut state: ResMut<NextState<TestState>>,
                | {
                    info!("1");
                    let root = query.single().unwrap();
                commands.entity(root).with_children(|s| {
                    s.spawn(Name::new("a"));
                    s.spawn(Name::new("b"));
                    s.spawn(Name::new("c"));
                });
                let _ = std::fs::create_dir_all(ASSET_DIR);
                commands.trigger(SaveSceneEvent(PathBuf::from(ASSET_PATH)));
                state.set(TestState::AwaitingLoad);
            })
            .add_systems(
                OnEnter(SaveStatus::Complete),
                (|
                    mut commands: Commands,
                    mut next_state: ResMut<NextState<TestState>>
                  | {
                      info!("2");
                        commands.trigger(LoadSceneEvent(PathBuf::from(ASSET_PATH)));
                        next_state.set(TestState::AwaitingLoad);
                })
                .run_if(in_state(TestState::AwaitingSave)),
            )
            .add_systems(
                OnEnter(LoadStatus::Complete),
                (|query: Query<&DynamicSceneRoot>, dyscenes: Res<Assets<DynamicScene>>| {
                    info!("3");
                    let scene = &query.single().unwrap().0;
                    let scene = dyscenes.get(scene).unwrap();
                    let entities = &scene.entities.iter().map(|e| &e.components).collect::<Vec<_>>();
                    let resources = &scene.resources;
                    info!("scene: {{\nentities: {entities:#?}\nresources: {resources:#?}\n}}");
                })
                .run_if(in_state(TestState::AwaitingLoad)),
            );
        app.run()
    })
    .with_timeout(10.)
    .run()
}
