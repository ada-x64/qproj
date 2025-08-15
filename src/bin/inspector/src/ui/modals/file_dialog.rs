// ------------------------------------------
// SPDX-License-Identifier: MIT OR Apache-2.0
// -------------------------------- 𝒒𝒑𝒓𝒐𝒋 --

#[derive(Debug, Default)]
pub enum UiFileState {
    #[default]
    None,
    SavingScene,
    LoadingScene,
    SavingLayout,
    LoadingLayout,
}
