mod app;
mod screen;
pub mod prelude {
    pub use super::app::prelude::*;
    pub use super::globals::*;
}

mod globals {
    use crate::{app::AppPlugin, prelude::EmptyScreen};
    use bevy::prelude::*;
    use bevy_asset::{AssetLoader, AsyncReadExt};
    use bevy_test_harness::{TestRunnerPlugin, TestRunnerTimeout};
    use tfw::{TfwPlugin, prelude::Screen};

    pub fn get_test_app<InitialScreen: Screen>() -> App {
        let mut app = App::new();
        app.add_plugins((
            TestRunnerPlugin::default(),
            AppPlugin,
            TfwPlugin(tfw::TfwSettings {
                initial_screen: InitialScreen::name(),
            }),
        ));
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
            reader: &mut dyn bevy_asset::io::Reader,
            _settings: &Self::Settings,
            _load_context: &mut bevy_asset::LoadContext<'a>,
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
