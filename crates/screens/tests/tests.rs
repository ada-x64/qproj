mod screens;
pub mod prelude {
    pub use super::globals::*;
    pub use super::screens::prelude::*;
    pub use bevy::prelude::*;
    pub use bevy_asset_loader::prelude::*;
    pub use q_screens::prelude::*;
    pub use q_test_harness::prelude::*;
}

mod globals {
    use crate::prelude::*;
    use bevy::asset::{AssetLoader, AsyncReadExt};

    pub fn get_test_app<S: Screen>() -> App {
        let mut app = App::new();
        app.add_plugins((TestRunnerPlugin::default(), ScreenPlugin));
        app.register_screen::<S>();
        app.insert_resource(InitialScreen::new::<S>());
        app.init_asset::<TextAsset>();
        app.init_asset_loader::<TextAssetLoader>();
        app
    }

    #[derive(Asset, Deref, DerefMut, Debug, Reflect)]
    pub struct TextAsset(String);

    #[derive(Reflect, Default)]
    pub struct TextAssetLoader;
    impl AssetLoader for TextAssetLoader {
        type Asset = TextAsset;
        type Settings = ();
        type Error = String;

        async fn load<'a>(
            &self,
            reader: &mut dyn bevy::asset::io::Reader,
            _settings: &Self::Settings,
            _load_context: &mut bevy::asset::LoadContext<'a>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut string = String::new();

            match reader.read_to_string(&mut string).await {
                Ok(_) => Ok(TextAsset(string)),
                Err(e) => Err(e.to_string()),
            }
        }

        fn extensions(&self) -> &[&str] {
            &["txt"]
        }
    }

    #[derive(Resource, Default)]
    struct TheHandle(Handle<TextAsset>);
    #[test]
    fn test_load() {
        let mut app = get_test_app::<EmptyScreen>();
        app.insert_resource(TestRunnerTimeout(0.25));
        app.init_resource::<TheHandle>();
        app.add_systems(
            Startup,
            |server: Res<AssetServer>, mut thehandle: ResMut<TheHandle>| {
                let handle = server.load("test/test.txt");
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
