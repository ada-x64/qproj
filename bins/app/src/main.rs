// 𝒒𝒑𝒓𝒐𝒋-- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use bevy::prelude::*;
use q_app::GameAppPlugin;

#[bevy_main]
fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, GameAppPlugin::default()))
        .run()
}
