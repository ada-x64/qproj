use bevy::{
    log::{tracing_subscriber::EnvFilter, LogPlugin},
    prelude::*,
};

#[bevy_main]
fn main() {
    let mut app = App::new();

    // use RUST_LOG
    app.add_plugins((
        DefaultPlugins.set(LogPlugin {
            filter: EnvFilter::from_default_env().to_string(),
            ..Default::default()
        }),
        worldgen::WorldgenPlugin {
            spawn_immediately: true,
        },
        player::PlayerPlugin {
            enable_flycam: true,
        },
    ));

    #[cfg(feature = "debug")]
    {
        use debug_gizmos::{DebugLevel, DebugPlugin};
        let level = std::env::var("DEBUG_LEVEL").unwrap_or_default();
        let debug_level = DebugLevel(level.parse().unwrap_or_default());
        debug!("DEBUG_LEVEL = {debug_level:?}");
        app.add_plugins(DebugPlugin { debug_level });
    }

    app.run();
}
