use bevy::prelude::*;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy::scene::ScenePlugin>() {
            app.add_plugins(bevy::scene::ScenePlugin);
        }
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((Name::new("Scene Root"), SceneRoot::default()));
        });
    }
}
