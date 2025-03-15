//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    log::{tracing_subscriber::EnvFilter, LogPlugin},
    prelude::*,
};
use player::{bevy_flycam::FlyCam, Player};
use worldgen::{util::SpawnAroundTracker, WorldgenPluginSettings};

fn setup(
    mut commands: Commands,
    mut query: Single<(&mut Transform, Entity), With<FlyCam>>,
) {
    commands.entity(query.1).insert((
        Player,
        SpawnAroundTracker,
        Name::new("Player"),
    ));
    query.0.translation.y = 200.;
    query.0.look_at(Vec3::ZERO, Vec3::Y);
    debug!("Post setup! Added player and spawnaroundtracker");
}

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
        player::PlayerPlugin,
    ))
    .insert_resource(WorldgenPluginSettings {
        spawn_immediately: true,
        use_debug_colors: cfg!(feature = "debug"),
    })
    .add_systems(PostStartup, setup);

    #[cfg(feature = "debug")]
    {
        use debug_gizmos::{DebugLevel, DebugPlugin};
        let level = std::env::var("DEBUG_LEVEL").unwrap_or_default();
        let debug_level = DebugLevel(level.parse().unwrap_or_default());
        debug!("DEBUG_LEVEL = {debug_level:?}");
        app.add_plugins(DebugPlugin {
            debug_level,
            wireframes: false,
        });
    }

    #[cfg(feature = "inspector")]
    {
        app.add_plugins(inspector::InspectorPlugin);
    }

    app.run();
}
