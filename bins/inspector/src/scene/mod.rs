use std::path::PathBuf;

use bevy::{ecs::world::CommandQueue, prelude::*, tasks::IoTaskPool};
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

#[derive(Event)]
pub struct SaveSceneEvent(pub PathBuf);
fn save_scene(trigger: Trigger<SaveSceneEvent>, world: &mut World) {
    let mut path = trigger.0.clone();
    path.set_extension("scn.ron");
    debug!("save_scene");
    task!(IoTaskPool, async move |q: &mut CommandQueue| {
        q.push(|world: &mut World| {
            debug!("Serializing scene...");
            let mut scene = world.query::<&DynamicSceneRoot>();
            let scene = scene.single(world);
            if let Err(e) = scene {
                Toast::Error.from_world(world, e.to_string());
                return;
            }
            let scene = scene.unwrap();
            let default_scene = DynamicScene::default();
            let scene = r!(world.get_resource::<Assets<DynamicScene>>())
                .get(scene.id())
                .unwrap_or(&default_scene);

            let serialized_scene = {
                let type_registry = r!(world.get_resource::<AppTypeRegistry>());
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
                let res = std::fs::write(&path, serialized_scene.as_bytes());
                match res {
                    Err(e) => Toast::Error.from_queue(q, e.to_string()),
                    Ok(_) => Toast::Success
                        .from_queue(q, format!("Saved file to {path:#?}")),
                }
            })(world);
        });
    })(world)
}
