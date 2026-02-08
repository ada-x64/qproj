use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    platform::collections::HashMap,
};

use crate::prelude::*;

/// Environment variables for this console. Saved as '.env' files on disk.
#[derive(Asset, Default, Component, Debug, Deref, DerefMut, Reflect)]
pub struct ConsoleEnvVars(HashMap<String, String>);

/// Wrapper for the [Handle] of the [ConsoleEnvVarsAsset] for this [Console]
#[derive(Component, Debug, Deref, DerefMut, Reflect)]
pub struct ConsoleEnvVarsHandle(pub Handle<ConsoleEnvVars>);

/// Loader for the [ConsoleEnvVarsAsset]
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

/// Command history of this [Console].
#[derive(Default, Asset, Debug, Deref, DerefMut, Reflect)]
pub struct ConsoleHistory(Vec<String>);

/// Wrapper around the [Handle] for the [ConsoleHistoryAsset] for this [Console]
#[derive(Component, Debug, Deref, DerefMut, Reflect, Clone)]
pub struct ConsoleHistoryHandle(pub Handle<ConsoleHistory>);

/// Loader for the [ConsoleHistoryAsset]
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
        let vec = buf.split('\n').map(|s| s.to_owned()).collect::<Vec<_>>();
        Ok(ConsoleHistory(vec))
    }

    fn extensions(&self) -> &[&str] {
        &["hist"]
    }
}

pub fn plugin(app: &mut App) {
    app.register_asset_loader(ConsoleHistoryLoader);
    app.init_asset::<ConsoleHistory>();
    app.register_asset_loader(ConsoleEnvVarsLoader);
    app.init_asset::<ConsoleEnvVars>();
}
