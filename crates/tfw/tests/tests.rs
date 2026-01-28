mod app;
mod screen;
pub use app::prelude;

mod globals {
    use bevy::{ecs::schedule::ExecutorKind, prelude::*};
    use bevy_test_harness::{TestRunnerPlugin, TestRunnerTimeout};
    #[derive(Resource, Default)]
    struct TheHandle(Handle<Image>);
    #[test]
    fn test_load_asset() {
        let mut app = App::new();
        app.add_plugins(TestRunnerPlugin {
            executor_kind: ExecutorKind::MultiThreaded,
            ..Default::default()
        });
        app.insert_resource(TestRunnerTimeout(0.25));
        app.init_resource::<TheHandle>();
        app.add_systems(
            Startup,
            |server: Res<AssetServer>, mut thehandle: ResMut<TheHandle>| {
                let handle = server.load("test/test.png");
                thehandle.0 = handle;
            },
        );
        app.add_systems(
            Update,
            |server: Res<AssetServer>, thehandle: Res<TheHandle>, mut commands: Commands| {
                match server.load_state(thehandle.0.id()) {
                    bevy::asset::LoadState::NotLoaded => {
                        warn!("Asset not started to load.");
                    }
                    bevy::asset::LoadState::Loading => {
                        info!("loading...");
                    }
                    bevy::asset::LoadState::Loaded => {
                        commands.write_message(AppExit::Success);
                    }
                    bevy::asset::LoadState::Failed(asset_load_error) => {
                        error!(?asset_load_error);
                        commands.write_message(AppExit::error());
                    }
                }
            },
        );
        assert!(app.run().is_success())
    }
}
