use crate::prelude::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Reflect, Default, Resource)]
pub struct NamedEntityScreenSettings {
    pub entity_name: String,
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct NamedEntityScreen;
impl Screen for NamedEntityScreen {
    type SETTINGS = NamedEntityScreenSettings;
}

fn init(settings: Res<NamedEntityScreenSettings>, mut commands: Commands) {
    commands.spawn(Name::new(settings.entity_name.clone()));
}

pub fn plugin(app: &mut App) {
    ScreenScopeBuilder::<NamedEntityScreen>::new(app)
        .on_ready(init)
        .build();
}
