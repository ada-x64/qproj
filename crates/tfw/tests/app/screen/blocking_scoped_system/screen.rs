use crate::prelude::*;

#[derive(PartialEq, Eq, Clone, Debug, Hash, Reflect, Default, Resource)]
pub struct BlockingScopedSystemSettings {
    pub initial_value: u32,
    pub unload_value: u32,
}
#[derive(PartialEq, Eq, Clone, Debug, Hash, Reflect, Default, Resource, Deref, DerefMut)]
pub struct BlockingScopedSystemValue(pub u32);

#[derive(AssetCollection, Resource, Debug, Default)]
pub struct BlockingScopedSystemAssets {
    #[asset(path = "test/test.txt")]
    _img: Handle<TextAsset>,
}

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub struct BlockingScopedSystemScreen;
impl Screen for BlockingScopedSystemScreen {
    type SETTINGS = BlockingScopedSystemSettings;
}

fn increment(mut value: ResMut<BlockingScopedSystemValue>) {
    **value += 1;
    info!("increment, value={}", **value);
}

fn init(mut value: ResMut<BlockingScopedSystemValue>, settings: Res<BlockingScopedSystemSettings>) {
    **value = settings.initial_value;
    info!("init, value={}", **value);
}
fn unload(
    mut value: ResMut<BlockingScopedSystemValue>,
    settings: Res<BlockingScopedSystemSettings>,
) {
    **value = settings.unload_value;
    info!("unload, value={}", **value);
}

pub fn plugin(app: &mut App) {
    ScreenScopeBuilder::<BlockingScopedSystemScreen>::new(app)
        .add_systems(ScreenSchedule::Update, increment)
        .with_load_strategy(LoadStrategy::Blocking) // default value
        .on_ready(init)
        .on_unload(unload)
        .build();
    app.init_resource::<BlockingScopedSystemValue>();
}
