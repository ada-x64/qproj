use bevy::app::HierarchyPropagatePlugin;

use crate::prelude::*;

#[derive(Default, Resource, Reflect, Debug)]
pub struct TfwSettings {
    /// Prefer to initialize this with [Self::with_initial_screen]
    pub initial_screen_name: Option<String>,
}
impl TfwSettings {
    pub fn with_initial_screen<S: Screen>() -> Self {
        Self {
            initial_screen_name: Some(S::name()),
        }
    }
}

/// The main export plugin for TFW. `Screens` should be an enum with screen
/// names. Refer to the template documentation for more details.
/// The template parameter refers to the initial screen.
#[derive(Default, Debug)]
pub struct TfwPlugin;
impl Plugin for TfwPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TfwSettings>();
        app.add_message::<SwitchToScreenMsg>();
        app.add_systems(
            Startup,
            |mut commands: Commands, settings: Res<TfwSettings>, registry: Res<ScreenRegistry>| {
                if let Some(initial_screen) = settings.initial_screen_name.clone() {
                    if let Some(cid) = registry
                        .values()
                        .find_map(|v| (v.name() == initial_screen).then_some(v.id()))
                    {
                        commands.write_message(SwitchToScreenMsg(cid));
                    } else {
                        warn!("Could not find screen with name {initial_screen}");
                    }
                }
            },
        );
        app.add_plugins((
            HierarchyPropagatePlugin::<Persistent>::new(PostUpdate),
            HierarchyPropagatePlugin::<ScreenScoped>::new(PostUpdate),
        ));
        app.add_plugins(super::systems::plugin);
    }
}
