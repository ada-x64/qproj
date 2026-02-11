use bevy::{
    asset::{AssetLoadError, AssetLoader, AsyncReadExt, io::embedded::GetAssetServer},
    ecs::{component::Mutable, lifecycle::HookContext, world::DeferredWorld},
    platform::collections::HashMap,
};

use crate::prelude::*;

mod assets_impl {
    use super::*;

    /// Environment variables for this console. Saved as '.env' files on disk.
    /// Follows conventional '.env' format.
    #[derive(Asset, Default, Component, Debug, Deref, DerefMut, Reflect, Clone, PartialEq)]
    pub struct ConsoleEnvVars(pub HashMap<String, String>);

    /// Command history of this [Console]. Saved as '.history' files on disk.
    /// Simple line-separated list of executed commands.
    #[derive(Default, Asset, Debug, Deref, DerefMut, Reflect, Clone, PartialEq)]
    pub struct ConsoleHistory(pub Vec<String>);
}
pub use assets_impl::*;

mod wrappers {
    use bevy::asset::{AsAssetId, AssetLoadFailedEvent};

    use super::*;

    /// A handle to some asset. In particular, this is used with [ConsoleEnvVars] and [ConsoleHistory]
    #[derive(Component, Debug, Reflect, Default, Clone)]
    #[component(on_insert=Self::on_insert)]
    pub struct ConsoleAssetHandle<A: Asset + Default> {
        path: Option<String>,
        handle: Handle<A>,
    }
    impl<A: Asset + Default> ConsoleAssetHandle<A> {
        pub fn new(path: String) -> Self {
            Self {
                path: Some(path),
                handle: Handle::default(),
            }
        }

        pub fn from_handle(handle: Handle<A>) -> Self {
            Self { path: None, handle }
        }

        pub fn path(&self) -> Option<&String> {
            self.path.as_ref()
        }

        pub fn handle(&self) -> &Handle<A> {
            &self.handle
        }

        fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
            let handle = {
                let server = world.get_asset_server();
                let this = world.get::<Self>(ctx.entity).unwrap();
                if let Some(path) = this.path() {
                    server.load(path.clone())
                } else {
                    server.add(A::default())
                }
            };
            let mut this = world.get_mut::<Self>(ctx.entity).unwrap();
            this.handle = handle;
        }
    }
    impl<A: Asset + Default> AutoCreateAsset for ConsoleAssetHandle<A> {
        type Target = A;
        fn set_handle(&mut self, handle: Handle<A>) {
            self.handle = handle
        }
        fn id(&self) -> AssetId<Self::Target> {
            self.handle.id()
        }
    }
    impl<A: Asset + Default> AsAssetId for ConsoleAssetHandle<A> {
        type Asset = A;
        fn as_asset_id(&self) -> AssetId<A> {
            self.handle.id()
        }
    }

    /// This trait will automatically create an asset on disk if it does not exist at load time.
    pub trait AutoCreateAsset: Component<Mutability = Mutable> + Sized {
        type Target: Asset + Default;

        fn set_handle(&mut self, handle: Handle<Self::Target>);
        fn id(&self) -> AssetId<Self::Target>;

        fn check_assets(
            mut reader: MessageReader<AssetLoadFailedEvent<Self::Target>>,
            mut these: Query<&mut Self>,
            server: Res<AssetServer>,
        ) {
            for val in reader.read() {
                if let AssetLoadError::AssetReaderError(_) = val.error {
                    // TODO: try writing an empty version to the path.
                    warn!(
                        "Failed to load path {:?}
    NOTE: Eventually, this will result in the asset being automatically created on disk or in the server.
    However, this relies on the AssetSaver struct, which is slated to release in Bevy 0.19.
    For now, the asset is added with its default value, but _not_ saved to disk.", val.path
                    );
                    if let Some(mut this) = these.iter_mut().find(|this| this.id() == val.id) {
                        this.set_handle(server.add(Self::Target::default()));
                    }
                }
            }
        }
    }
}
pub use wrappers::*;

mod loaders {
    use super::*;

    /// Loader for [ConsoleEnvVars]
    #[derive(Reflect, Default, Debug)]
    pub struct ConsoleEnvVarsLoader;
    impl AssetLoader for ConsoleEnvVarsLoader {
        type Asset = ConsoleEnvVars;
        type Settings = ();
        type Error = BevyError;

        async fn load(
            &self,
            reader: &mut dyn bevy::asset::io::Reader,
            _settings: &(),
            load_context: &mut bevy::asset::LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut buf = String::new();
            reader.read_to_string(&mut buf).await?;
            let map = buf
                .split('\n')
                .filter_map(|s| {
                    let mut split = s.split('=').map(|s| s.to_string()).collect::<Vec<_>>();
                    if split.len() == 2 {
                        Some((std::mem::take(&mut split[0]), std::mem::take(&mut split[1])))
                    } else {
                        warn!(
                            "Got invalid line while reading {}:\n'{}'",
                            load_context.path(),
                            s
                        );
                        None
                    }
                })
                .collect::<HashMap<_, _>>();
            Ok(ConsoleEnvVars(map))
        }

        fn extensions(&self) -> &[&str] {
            &["env"]
        }
    }

    /// Loader for [ConsoleHistory]
    #[derive(Reflect, Default, Debug)]
    pub struct ConsoleHistoryLoader;
    impl AssetLoader for ConsoleHistoryLoader {
        type Asset = ConsoleHistory;
        type Settings = ();
        type Error = BevyError;

        async fn load(
            &self,
            reader: &mut dyn bevy::asset::io::Reader,
            _settings: &(),
            _load_context: &mut bevy::asset::LoadContext<'_>,
        ) -> Result<Self::Asset, Self::Error> {
            let mut buf = String::new();
            reader.read_to_string(&mut buf).await?;
            // todo: not memory efficient
            let vec = buf
                .split('\n')
                .filter_map(|s| (!s.is_empty()).then_some(s.to_owned()))
                .collect::<Vec<_>>();
            Ok(ConsoleHistory(vec))
        }

        fn extensions(&self) -> &[&str] {
            &["hist"]
        }
    }
}
pub use loaders::*;

pub fn plugin(app: &mut App) {
    app.register_asset_loader(ConsoleHistoryLoader);
    app.init_asset::<ConsoleHistory>();
    app.register_asset_loader(ConsoleEnvVarsLoader);
    app.init_asset::<ConsoleEnvVars>();
    app.add_systems(
        PreUpdate,
        ConsoleAssetHandle::<ConsoleEnvVars>::check_assets,
    );
    app.add_systems(
        PreUpdate,
        ConsoleAssetHandle::<ConsoleHistory>::check_assets,
    );
}

#[cfg(test)]
mod test {
    use bevy::asset::AssetLoadFailedEvent;

    use super::*;

    fn test_asset_load<T: Asset + Default + PartialEq, M>(
        path: String,
        callback: impl IntoSystem<In<AssetId<T>>, (), M> + 'static,
    ) {
        let mut app = App::new();
        app.add_plugins(crate::test_harness::plugin);
        let callback = app.register_system(callback);
        app.add_systems(Startup, move |mut commands: Commands| {
            info!("spawning...");
            commands.spawn(ConsoleAssetHandle::<T>::new(path.to_string()));
        });
        app.add_systems(
            Update,
            |mut commands: Commands,
             mut reader: MessageReader<AssetLoadFailedEvent<T>>,
             this: Query<&ConsoleAssetHandle<T>>| {
                info!("reading fail msgs...");
                let this = this.single().unwrap();
                for msg in reader.read() {
                    if this.handle().id() == msg.id
                        && let AssetLoadError::AssetReaderError(asset_reader_error) = &msg.error
                    {
                        error!(?asset_reader_error);
                        commands.write_message(AppExit::error());
                    }
                }
            },
        );
        app.add_systems(
            Update,
            move |mut reader: MessageReader<AssetEvent<T>>,
                  this: Query<&ConsoleAssetHandle<T>>,
                  assets: Res<Assets<T>>,
                  mut commands: Commands| {
                info!("reading event msgs...");
                let this = this.single().unwrap();
                for msg in reader.read() {
                    debug!(?msg);
                    if let AssetEvent::LoadedWithDependencies { id } = msg
                        && this.handle().id() == *id
                    {
                        let handle = this.handle();
                        if let Some(asset) = assets.get(handle)
                            && *asset != T::default()
                        {
                            commands.run_system_with(callback, *id);
                        }
                    }
                }
            },
        );
        assert!(app.run().is_success())
    }

    #[test]
    fn test_env_var_load() {
        test_asset_load(
            "console.env".to_string(),
            |input: In<AssetId<ConsoleEnvVars>>,
             assets: Res<Assets<ConsoleEnvVars>>,
             mut commands: Commands| {
                let asset = assets.get(*input).unwrap();
                info!(?asset);
                let mut ok = true;
                ok = ok && asset.get("FOO") == Some(&"BAR".to_string());
                ok = ok && asset.get("HI") == Some(&"HELLO".to_string());
                if ok {
                    commands.write_message(AppExit::Success);
                } else {
                    commands.write_message(AppExit::error());
                }
            },
        );
    }
    #[test]
    fn test_history_load() {
        test_asset_load(
            "console.history".to_string(),
            |input: In<AssetId<ConsoleHistory>>,
             assets: Res<Assets<ConsoleHistory>>,
             mut commands: Commands| {
                let asset = assets.get(*input).unwrap();
                info!(?asset);
                let ok = asset.0 == ["1", "2", "3", "4", "5"];
                if ok {
                    commands.write_message(AppExit::Success);
                } else {
                    commands.write_message(AppExit::error());
                }
            },
        );
    }
}
