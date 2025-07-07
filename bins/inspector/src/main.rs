//         •
// ┏┓┏┓┏┓┏┓┓
// ┗┫┣┛┛ ┗┛┃
//--┗┛-----┛------------------------------------------ (c) 2025 contributors ---
use bevy::{
    asset::{AssetMetaCheck, UnapprovedPathMode},
    prelude::*,
};
use q_inspector::{InspectorPlugin, prelude::InspectorState};

#[bevy_main]
fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            unapproved_path_mode: UnapprovedPathMode::Allow,
            watch_for_changes_override: Some(true),
            mode: AssetMode::Processed,
            meta_check: AssetMetaCheck::Always,
            ..Default::default()
        }),
        InspectorPlugin,
    ))
    .add_systems(
        Startup,
        |mut state: ResMut<NextState<InspectorState>>| {
            state.set(InspectorState::Enabled);
        },
    );
    app.run()
}
