//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::app::App;
use easy_ext::ext;

pub mod cam;
pub use cam::*;

#[ext(SetupComponents)]
pub impl App {
    fn setup_components(&mut self) -> &mut Self {
        self.setup_cam_component()
    }
}
