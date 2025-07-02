//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::prelude::*;
use q_app::GameAppPlugin;
use q_inspector::{InspectorPlugin, prelude::InspectorState};

#[bevy_main]
fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins,
        InspectorPlugin,
        // GameAppPlugin { main_app: false },
    ))
    .add_systems(
        Startup,
        |mut state: ResMut<NextState<InspectorState>>| {
            state.set(InspectorState::Enabled);
        },
    );
    app.run()
}
