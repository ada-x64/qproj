//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use q_app::GameAppPlugin;

#[bevy_main]
fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((DefaultPlugins, GameAppPlugin::default()))
        .run()
}
