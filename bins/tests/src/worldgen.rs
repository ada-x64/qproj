// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use q_worldgen::util::TerrainIntialized;

use crate::Runner;

#[test]
fn serialize() -> AppExit {
    Runner::new(|app| {
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
        app.run()
    })
    .run()
}
