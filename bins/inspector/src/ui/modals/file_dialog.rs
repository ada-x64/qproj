// 𝒒𝒑𝒓𝒐𝒋 -- copyright (c) the contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

#[derive(Debug, Default)]
pub enum UiFileState {
    #[default]
    None,
    SavingScene,
    LoadingScene,
    SavingLayout,
    LoadingLayout,
}
