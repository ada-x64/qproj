// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

use bevy::{
    asset::{AssetLoader, AsyncReadExt},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Asset, TypePath, Deref, Clone, Serialize, Deserialize)]
pub struct Expr(pub noise_expr::Expr);

#[derive(Error, Debug)]
pub enum ExprError {
    #[error("IO error {0}")]
    Io(#[from] std::io::Error),
    #[error("RON error {0}")]
    Ron(#[from] ron::de::SpannedError),
}

#[derive(Default)]
pub struct ExprLoader;
impl AssetLoader for ExprLoader {
    type Asset = Expr;
    type Settings = ();
    type Error = ExprError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut str = String::new();
        reader.read_to_string(&mut str).await?;
        let expr: noise_expr::Expr = ron::from_str(&str)?;
        Ok(Expr(expr))
    }
}
