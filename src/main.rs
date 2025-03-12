//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    log::{tracing_subscriber::EnvFilter, LogPlugin},
    prelude::*,
};
use worldgen::WorldgenPluginSettings;

#[bevy_main]
fn main() {
    let mut app = App::new();

    // use RUST_LOG
    app.add_plugins((
        DefaultPlugins.set(LogPlugin {
            filter: EnvFilter::from_default_env().to_string(),
            ..Default::default()
        }),
        worldgen::WorldgenPlugin,
        player::PlayerPlugin {
            enable_flycam: true,
        },
    ))
    .insert_resource(WorldgenPluginSettings {
        spawn_immediately: true,
        use_debug_colors: cfg!(feature = "debug"),
    });

    #[cfg(feature = "debug")]
    {
        use debug_gizmos::{DebugLevel, DebugPlugin};
        let level = std::env::var("DEBUG_LEVEL").unwrap_or_default();
        let debug_level = DebugLevel(level.parse().unwrap_or_default());
        debug!("DEBUG_LEVEL = {debug_level:?}");
        app.add_plugins(DebugPlugin {
            debug_level,
            wireframes: true,
        });
    }

    #[cfg(feature = "inspector")]
    {
        app.add_plugins(inspector::InspectorPlugin);
    }

    app.run();
}
