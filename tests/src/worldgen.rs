use crate::run_headless;
use bevy::prelude::*;
use q_worldgen::util::TerrainIntialized;

#[test]
fn serialize() -> AppExit {
    run_headless(|app| {
        app.add_plugins(q_worldgen::WorldgenPlugin);
        app.finish();
        app.cleanup();
        app.add_observer(|_: Trigger<TerrainIntialized>, world: &mut World| {
            {
                let scene = DynamicSceneBuilder::from_world(world).allow_all();
                let type_registry = world.resource::<AppTypeRegistry>();
                let type_registry = type_registry.read();
                let res = scene.build().serialize(&type_registry).unwrap();
                info!("{res}")
            }
            world.send_event(AppExit::Success);
        });
        info!("Running app.");
        let res = app.run();
        info!("Finished!");
        res
    })
}
