#[derive(Debug, Default)]
pub enum UiFileState {
    #[default]
    None,
    SavingScene,
    LoadingScene,
    SavingLayout,
    LoadingLayout,
}
