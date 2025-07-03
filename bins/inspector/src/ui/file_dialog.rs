#[derive(Debug)]
pub enum FileType {
    SaveScene,
    LoadScene,
    SaveLayout,
    LoadLayout,
}

#[derive(Debug, Default)]
pub enum UiFileState {
    #[default]
    None,
    SavingScene,
    LoadingScene,
    SavingLayout,
    LoadingLayout,
}
