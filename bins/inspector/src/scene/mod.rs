use std::{any::TypeId, path::PathBuf};

use bevy::{
    ecs::world::CommandQueue, platform::collections::HashSet, prelude::*,
    tasks::IoTaskPool,
};
use q_tasks::task;
use tiny_bail::prelude::*;

use crate::{
    scene::{gizmos::GizmosPlugin, inspector_cam::InspectorCamPlugin},
    ui::modals::toast::Toast,
};

pub mod gizmos;
pub mod inspector_cam;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::scene::ScenePlugin>() {
            app.add_plugins(bevy::scene::ScenePlugin);
        }
        app.add_plugins((InspectorCamPlugin, GizmosPlugin));
        app.add_systems(Startup, |mut commands: Commands| {
            commands
                .spawn((Name::new("Scene Root"), DynamicSceneRoot::default()));
        })
        .add_event::<SaveSceneEvent>()
        .add_observer(save_scene);
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
    path.set_extension("scn.ron");
    task!(IoTaskPool, async move |q: &mut CommandQueue| {
        q.push(move |world: &mut World| {
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
                    .with_resource_filter(SceneFilter::Denylist(
                        HashSet::from_iter([TypeId::of::<Time>()]),
                    ));
                extract(builder, world, scene)
            };
            let serialized_scene = {
                let serialized_scene = {
                    let type_registry =
                        r!(world.get_resource::<AppTypeRegistry>());
                    builder.build().serialize(&type_registry.read())
                };
                if let Err(e) = serialized_scene {
                    Toast::Error.from_world(world, e.to_string());
                    return;
                }
                serialized_scene.unwrap()
            };

            let res = std::fs::write(&path, serialized_scene.as_bytes());
            match res {
                Err(e) => Toast::Error.from_world(world, e.to_string()),
                Ok(_) => Toast::Success
                    .from_world(world, format!("Saved file to {path:#?}")),
            };
        });
    })(world)
}
